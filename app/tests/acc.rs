#![feature(async_fn_track_caller)]

mod acc {
    mod admin;
    mod challenges;
    mod game;
    mod index;
    mod tutorial;
}

use anyhow::{Error, Result};
use avt::Vt;
use base64::Engine;
use flate2::read::GzDecoder;
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use kartoffels_store::{Session, SessionId, Store, World};
use kartoffels_utils::Asserter;
use kartoffels_world::prelude::Handle as WorldHandle;
use russh::keys::ssh_key::PublicKey;
use russh::{client as ssh, ChannelId};
use std::io::{Cursor, Read};
use std::mem;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use termwiz::input::{
    KeyCode, KeyCodeEncodeModes, KeyboardEncoding, Modifiers,
};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::{self, JoinHandle};
use tokio::time;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_util::sync::CancellationToken;
use tungstenite::Error as WsError;

type Stdin = Box<dyn Sink<WsMessage, Error = WsError> + Unpin>;
type Stdout = Box<dyn Stream<Item = Result<WsMessage, WsError>> + Unpin>;

struct TestContext {
    store: Arc<Store>,

    admin_addr: SocketAddr,
    admin_server: JoinHandle<Result<()>>,

    http_addr: SocketAddr,
    http_server: JoinHandle<Result<()>>,

    term: Vt,
    stdin: Stdin,
    stdout: Stdout,
}

impl TestContext {
    pub const INDEX: &str =
        "welcome to kartoffels, a game where you're given a potato";

    pub async fn new(worlds: impl IntoIterator<Item = WorldHandle>) -> Self {
        let mut this = Self::new_ex(80, 30, worlds).await;

        this.wait_for(Self::INDEX).await;
        this
    }

    async fn new_ex(
        cols: usize,
        rows: usize,
        worlds: impl IntoIterator<Item = WorldHandle>,
    ) -> Self {
        _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let store = Self::open_store(worlds).await;
        let shutdown = CancellationToken::new();

        let (admin_addr, admin_server) =
            Self::start_admin_server(store.clone(), shutdown.clone()).await;

        let (http_addr, http_server, stdin, stdout) =
            Self::start_http_server(cols, rows, store.clone(), shutdown).await;

        Self {
            store,
            admin_addr,
            admin_server,
            http_addr,
            http_server,
            term: Vt::new(cols, rows),
            stdin,
            stdout,
        }
    }

    async fn start_admin_server(
        store: Arc<Store>,
        shutdown: CancellationToken,
    ) -> (SocketAddr, JoinHandle<Result<()>>) {
        let socket = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let addr = socket.local_addr().unwrap();

        let server = task::spawn(kartoffels_server::admin::start(
            None, socket, store, shutdown,
        ));

        (addr, server)
    }

    async fn open_store(
        worlds: impl IntoIterator<Item = WorldHandle>,
    ) -> Arc<Store> {
        let store = Store::open(None, true).await.unwrap();

        for world in worlds {
            store.add_world(world).await.unwrap();
        }

        Arc::new(store)
    }

    async fn start_http_server(
        cols: usize,
        rows: usize,
        store: Arc<Store>,
        shutdown: CancellationToken,
    ) -> (SocketAddr, JoinHandle<Result<()>>, Stdin, Stdout) {
        let socket = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let addr = socket.local_addr().unwrap();

        let server = task::spawn(kartoffels_server::http::start(
            socket,
            store.clone(),
            shutdown,
        ));

        let client = time::timeout(Duration::from_secs(1), async move {
            loop {
                time::sleep(Duration::from_millis(1)).await;

                let conn =
                    tokio_tungstenite::connect_async(format!("ws://{addr}"))
                        .await;

                if let Ok((client, _)) = conn {
                    break client;
                }
            }
        })
        .await
        .unwrap();

        let (mut stdin, mut stdout) = client.split();

        stdin
            .send(WsMessage::Text(
                format!("{{ \"cols\": {cols}, \"rows\": {rows} }}").into(),
            ))
            .await
            .unwrap();

        stdout.next().await.unwrap().unwrap();

        (addr, server, Box::new(stdin), Box::new(stdout))
    }

    pub fn asserter(&self, dir: impl AsRef<Path>) -> Asserter {
        Asserter::new(Path::new("tests").join("acc").join(dir))
    }

    pub async fn recv(&mut self) {
        let compressed_stdout =
            self.stdout.next().await.unwrap().unwrap().into_data();

        let mut stdout = Vec::new();

        GzDecoder::new(Cursor::new(compressed_stdout))
            .read_to_end(&mut stdout)
            .unwrap();

        self.term.feed_str(&String::from_utf8_lossy(&stdout));
    }

    pub async fn press(&mut self, key: KeyCode) {
        self.press_ex(key, Modifiers::NONE).await;
    }

    pub async fn press_ex(&mut self, key: KeyCode, mods: Modifiers) {
        let payload: Vec<_> = {
            let modes = KeyCodeEncodeModes {
                encoding: KeyboardEncoding::Xterm,
                application_cursor_keys: false,
                newline_mode: false,
                modify_other_keys: None,
            };

            let is_down = true;

            key.encode(mods, modes, is_down).unwrap().into()
        };

        self.stdin
            .send(WsMessage::Binary(payload.into()))
            .await
            .unwrap();
    }

    #[track_caller]
    pub async fn sync(&mut self, version: u64) {
        self.wait_for(&format!("v{version}")).await;
    }

    #[track_caller]
    pub async fn wait_for(&mut self, text: &str) {
        let result = time::timeout(Duration::from_secs(1), async {
            while !self.stdout().contains(text) {
                self.recv().await;
            }
        })
        .await;

        if result.is_err() {
            panic!(
                "wait_for(\"{text}\") failed, stdout was:\n\n{}",
                self.stdout()
            );
        }
    }

    #[track_caller]
    pub async fn wait_for_window(&mut self, title: &str) {
        self.wait_for(&format!("─ {title} ─")).await;
    }

    #[track_caller]
    pub async fn wait_while(&mut self, text: &str) {
        let result = time::timeout(Duration::from_secs(1), async {
            while self.stdout().contains(text) {
                self.recv().await;
            }
        })
        .await;

        if result.is_err() {
            panic!(
                "wait_while(\"{text}\") failed, stdout was:\n\n{}",
                self.stdout()
            );
        }
    }

    #[track_caller]
    pub async fn wait_while_modal(&mut self, title: &str) {
        self.wait_while(&format!("─ {title} ─")).await;
    }

    #[track_caller]
    pub fn see(&mut self, text: &str) {
        let stdout = self.stdout();

        if !stdout.contains(text) {
            panic!("see(\"{text}\") failed, stdout was:\n\n{stdout}");
        }
    }

    #[track_caller]
    pub fn see_frame(&mut self, expected: impl AsRef<Path>) {
        let expected = expected.as_ref();
        let expected_dir = expected.parent().unwrap();
        let expected_file = expected.file_name().unwrap();

        self.asserter(expected_dir)
            .assert(expected_file, self.stdout());
    }

    #[track_caller]
    pub fn dont_see(&mut self, text: &str) {
        let stdout = self.stdout();

        if stdout.contains(text) {
            panic!("dont_see(\"{text}\") failed, stdout was:\n\n{stdout}");
        }
    }

    pub async fn upload_bot(&mut self, payload: &[u8]) {
        let src = base64::engine::general_purpose::STANDARD.encode(payload);

        let bracketed_paste_beg = "\x1b[200~";
        let bracketed_paste_end = "\x1b[201~";

        let payload = bracketed_paste_beg
            .bytes()
            .chain(src.bytes())
            .chain(bracketed_paste_end.bytes())
            .collect();

        self.press(KeyCode::Char('u')).await;
        self.stdin.send(WsMessage::Binary(payload)).await.unwrap();
    }

    pub async fn upload_bot_http(&mut self, sess: SessionId, src: &[u8]) {
        let url = format!("http://{}/sessions/{sess}/bots", self.http_addr);

        reqwest::Client::new()
            .post(url)
            .body(src.to_owned())
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap();
    }

    pub async fn world(&self) -> World {
        self.store
            .find_worlds(None)
            .await
            .unwrap()
            .into_iter()
            .next()
            .unwrap()
    }

    pub async fn session(&self) -> Session {
        self.store
            .find_sessions(None)
            .await
            .unwrap()
            .into_iter()
            .next()
            .unwrap()
    }

    pub fn stdout(&self) -> String {
        self.term.text().join("\n")
    }

    pub async fn cmd(&self, cmd: impl Into<String>) -> String {
        #[derive(Debug)]
        struct Client {
            tx: Option<oneshot::Sender<Vec<u8>>>,
            data: Vec<u8>,
        }

        impl ssh::Handler for Client {
            type Error = Error;

            async fn check_server_key(
                &mut self,
                _: &PublicKey,
            ) -> Result<bool> {
                Ok(true)
            }

            async fn data(
                &mut self,
                _: ChannelId,
                data: &[u8],
                _: &mut ssh::Session,
            ) -> Result<()> {
                self.data.extend(data);

                Ok(())
            }

            async fn channel_close(
                &mut self,
                _: ChannelId,
                _: &mut ssh::Session,
            ) -> Result<()> {
                _ = self.tx.take().unwrap().send(mem::take(&mut self.data));

                Ok(())
            }
        }

        let (tx, rx) = oneshot::channel();

        let mut conn = {
            let client = Client {
                tx: Some(tx),
                data: Vec::new(),
            };

            let config = Arc::new(ssh::Config::default());

            ssh::connect(config, &self.admin_addr, client)
                .await
                .unwrap()
        };

        conn.authenticate_none("").await.unwrap();

        conn.channel_open_session()
            .await
            .unwrap()
            .exec(true, cmd.into().as_bytes())
            .await
            .unwrap();

        String::from_utf8(rx.await.unwrap()).unwrap()
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        self.admin_server.abort();
        self.http_server.abort();
    }
}
