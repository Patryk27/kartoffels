#![feature(async_fn_track_caller)]

mod acceptance {
    mod challenges;
    mod game;
    mod home;
    mod tutorial;
}

use anyhow::Result;
use avt::Vt;
use base64::Engine;
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use kartoffels_store::{SessionId, Store};
use kartoffels_world::prelude::Handle as WorldHandle;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use termwiz::input::{
    KeyCode, KeyCodeEncodeModes, KeyboardEncoding, Modifiers,
};
use tokio::net::TcpListener;
use tokio::process::Command;
use tokio::task::{self, JoinHandle};
use tokio::{fs, time};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_util::sync::CancellationToken;
use tungstenite::Error as WsError;

struct TestContext {
    addr: SocketAddr,
    store: Arc<Store>,
    server: JoinHandle<Result<()>>,
    term: Vt,
    stdin: Box<dyn Sink<WsMessage, Error = WsError> + Unpin>,
    stdout: Box<dyn Stream<Item = Result<WsMessage, WsError>> + Unpin>,
}

impl TestContext {
    pub const HOME: &str =
        "welcome to kartoffels, a game where you're given a potato";

    pub async fn new(worlds: impl IntoIterator<Item = WorldHandle>) -> Self {
        Self::new_ex(80, 30, worlds).await
    }

    async fn new_ex(
        cols: usize,
        rows: usize,
        worlds: impl IntoIterator<Item = WorldHandle>,
    ) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let store = Arc::new(Store::test(worlds).await);
        let shutdown = CancellationToken::new();
        let addr = listener.local_addr().unwrap();

        let server = task::spawn(kartoffels_server::http::start(
            listener,
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
            .send(WsMessage::Text(format!(
                "{{ \"cols\": {cols}, \"rows\": {rows} }}",
            )))
            .await
            .unwrap();

        stdout.next().await.unwrap().unwrap();

        Self {
            addr,
            store,
            server,
            term: Vt::new(cols, rows),
            stdin: Box::new(stdin),
            stdout: Box::new(stdout),
        }
    }

    async fn recv(&mut self) {
        let stdout = self.stdout.next().await.unwrap().unwrap().into_data();

        for input in stdout {
            self.term.feed(input as char);
        }
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

        self.stdin.send(WsMessage::Binary(payload)).await.unwrap();
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
    pub async fn wait_for_modal(&mut self, text: &str) {
        self.wait_for(&format!("â {text} â")).await;
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
    pub async fn wait_while_modal(&mut self, text: &str) {
        self.wait_while(&format!("â {text} â")).await;
    }

    #[track_caller]
    pub async fn wait_for_change(&mut self) {
        let result = time::timeout(Duration::from_secs(1), async {
            let stdout = self.stdout();

            while self.stdout() == stdout {
                self.recv().await;
            }
        })
        .await;

        if result.is_err() {
            panic!(
                "wait_for_change() failed, stdout was:\n\n{}",
                self.stdout()
            );
        }
    }

    #[track_caller]
    pub fn see(&mut self, text: &str) {
        let stdout = self.stdout();

        if !stdout.contains(text) {
            panic!("see(\"{text}\") failed, stdout was:\n\n{stdout}");
        }
    }

    #[track_caller]
    pub async fn see_frame(&mut self, expected_path: &str) {
        let actual = self.stdout();

        let expected_path = format!("tests/acceptance/{expected_path}");

        let expected =
            fs::read_to_string(&expected_path).await.unwrap_or_default();

        let new_path = format!("{expected_path}.new");

        if actual == expected {
            _ = fs::remove_file(&new_path).await;
        } else {
            fs::write(&new_path, actual).await.unwrap();

            panic!("see_frame(\"{expected_path}\") failed");
        }
    }

    #[track_caller]
    pub fn dont_see(&mut self, text: &str) {
        let stdout = self.stdout();

        if stdout.contains(text) {
            panic!("not_see(\"{text}\") failed, stdout was:\n\n{stdout}");
        }
    }

    pub async fn upload_bot(&mut self, name: &str) {
        let payload = Self::build_bot(name).await;
        let payload = base64::engine::general_purpose::STANDARD.encode(payload);

        let bracketed_paste_beg = "\x1b[200~";
        let bracketed_paste_end = "\x1b[201~";

        let payload = bracketed_paste_beg
            .bytes()
            .chain(payload.bytes())
            .chain(bracketed_paste_end.bytes())
            .collect();

        self.press(KeyCode::Char('u')).await;
        self.stdin.send(WsMessage::Binary(payload)).await.unwrap();
    }

    pub async fn upload_bot_http(&mut self, sess: SessionId, name: &str) {
        let url = format!("http://{}/sessions/{sess}/bots", self.addr);
        let body = Self::build_bot(name).await;

        reqwest::Client::new()
            .post(url)
            .body(body)
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap();
    }

    pub fn store(&self) -> &Store {
        &self.store
    }

    pub fn stdout(&self) -> String {
        self.term.text().join("\n")
    }

    async fn build_bot(name: &str) -> Vec<u8> {
        let status = Command::new("just")
            .arg("bot")
            .arg(name)
            .status()
            .await
            .unwrap();

        if !status.success() {
            panic!("upload_bot(\"{name}\") failed");
        }

        fs::read(
            Path::new("target.riscv")
                .join("riscv64-kartoffel-bot")
                .join("release")
                .join(format!("bot-{name}")),
        )
        .await
        .unwrap()
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        self.server.abort();
    }
}
