mod acc {
    mod challenges;
    mod home;
    mod tutorial;
}

use anyhow::Result;
use avt::Vt;
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use kartoffels_store::Store;
use std::sync::Arc;
use std::time::Duration;
use termwiz::input::{
    KeyCode, KeyCodeEncodeModes, KeyboardEncoding, Modifiers,
};
use tokio::net::TcpListener;
use tokio::task::{self, JoinHandle};
use tokio::{fs, time};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_util::sync::CancellationToken;
use tungstenite::Error as WsError;

struct TestContext {
    server: JoinHandle<Result<()>>,
    term: Vt,
    stdin: Box<dyn Sink<WsMessage, Error = WsError> + Unpin>,
    stdout: Box<dyn Stream<Item = Result<WsMessage, WsError>> + Unpin>,
}

impl TestContext {
    pub const HOME: &str =
        "welcome to kartoffels, a game where you're given a potato";

    pub async fn new() -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let store = Arc::new(Store::test());
        let shutdown = CancellationToken::new();
        let addr = listener.local_addr().unwrap();

        let server = task::spawn(kartoffels_server::http::start(
            listener, store, shutdown,
        ));

        let client = time::timeout(Duration::from_secs(1), async move {
            loop {
                time::sleep(Duration::from_millis(1)).await;

                let conn =
                    tokio_tungstenite::connect_async(format!("ws://{}", addr))
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
            .send(WsMessage::Text(r#"{ "cols": 64, "rows": 32 }"#.into()))
            .await
            .unwrap();

        stdout.next().await.unwrap().unwrap();

        Self {
            server,
            term: Vt::new(64, 32),
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

    pub async fn upload_bot(&mut self, _id: &str) {
        todo!();
    }

    pub async fn wait_for(&mut self, text: &str) {
        let result = time::timeout(Duration::from_secs(1), async {
            while !self.stdout().contains(text) {
                self.recv().await;
            }
        })
        .await;

        if result.is_err() {
            let stdout = self.term.text().join("\n");

            panic!("wait_for(\"{text}\") failed, stdout was:\n\n{stdout}");
        }
    }

    pub fn see(&mut self, text: &str) {
        let stdout = self.stdout();

        if !stdout.contains(text) {
            panic!("see(\"{text}\") failed, stdout was:\n\n{stdout}");
        }
    }

    pub fn not_see(&mut self, text: &str) {
        let stdout = self.stdout();

        if stdout.contains(text) {
            panic!("not_see(\"{text}\") failed, stdout was:\n\n{stdout}");
        }
    }

    pub async fn assert(&mut self, expected_path: &str) {
        let actual = self.stdout();

        let expected =
            fs::read_to_string(expected_path).await.unwrap_or_default();

        let new_path = format!("{expected_path}.new");

        if actual == expected {
            _ = fs::remove_file(&new_path).await;
        } else {
            fs::write(&new_path, actual).await.unwrap();

            panic!("snapshot(\"{expected_path}\") failed");
        }
    }

    pub fn stdout(&self) -> String {
        self.term.text().join("\n")
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        self.server.abort();
    }
}
