use anyhow::{anyhow, Result};
use futures_util::sink::drain;
use futures_util::SinkExt;
use glam::uvec2;
use kartoffels_store::Store;
use kartoffels_ui::Term;
use russh::server::Session;
use russh::ChannelId;
use std::sync::Arc;
use termwiz::input::{InputEvent, InputParser, KeyCode, Modifiers};
use tokio::sync::mpsc;
use tokio::task;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tracing::info;

#[derive(Debug)]
pub enum AppChannel {
    AwaitingPty {
        store: Arc<Store>,
    },

    Ready {
        stdin_tx: mpsc::Sender<InputEvent>,
        stdin_parser: InputParser,
    },
}

impl AppChannel {
    pub fn new(store: Arc<Store>) -> Self {
        AppChannel::AwaitingPty { store }
    }

    pub async fn data(
        &mut self,
        id: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<()> {
        let AppChannel::Ready {
            stdin_tx,
            stdin_parser,
        } = self
        else {
            return Err(anyhow!("pty hasn't been allocated yet"));
        };

        for event in stdin_parser.parse_as_vec(data, false) {
            if let InputEvent::Key(event) = &event {
                if event.key == KeyCode::Char('c')
                    && event.modifiers == Modifiers::CTRL
                {
                    session.data(id, Term::exit_sequence().into_bytes().into());
                    session.close(id);

                    break;
                }
            }

            stdin_tx
                .send(event)
                .await
                .map_err(|_| anyhow!("ui thread has died"))?;
        }

        Ok(())
    }

    pub async fn pty_request(
        &mut self,
        id: ChannelId,
        width: u32,
        height: u32,
        session: &mut Session,
    ) -> Result<()> {
        let AppChannel::AwaitingPty { store } = self else {
            return Err(anyhow!("pty has been already allocated"));
        };

        let (stdin_tx, stdin_rx) = mpsc::channel(32);
        let stdin = Box::pin(ReceiverStream::new(stdin_rx).map(Ok));

        let stdout = Box::pin({
            let handle = session.handle();

            drain().with(move |stdout: Vec<u8>| {
                let handle = handle.clone();

                async move {
                    match handle.data(id, stdout.into()).await {
                        Ok(_) => Ok(()),
                        Err(_) => Err(anyhow!("got ssh channel overflow")),
                    }
                }
            })
        });

        let size = uvec2(width, height);
        let store = store.clone();
        let handle = session.handle();

        let mut term = Term::new(stdin, stdout, size)?;

        task::spawn(async move {
            let result = task::spawn(async move {
                kartoffels_ui::main(&mut term, &store).await
            })
            .await;

            _ = handle
                .data(id, Term::exit_sequence().into_bytes().into())
                .await;

            match result {
                Ok(result) => {
                    info!("ui task returned: {:?}", result);
                }

                Err(err) => {
                    _ = handle
                        .data(
                            id,
                            "whoopsie, the game has crashed, sorry!\r\n"
                                .to_string()
                                .into_bytes()
                                .into(),
                        )
                        .await;

                    info!("ui task crashed: {}", err);
                }
            }

            _ = handle.close(id).await;
        });

        *self = AppChannel::Ready {
            stdin_tx,
            stdin_parser: InputParser::default(),
        };

        Ok(())
    }

    pub async fn window_change_request(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<()> {
        let AppChannel::Ready { stdin_tx, .. } = self else {
            return Err(anyhow!("pty hasn't been allocated yet"));
        };

        stdin_tx
            .send(InputEvent::Resized {
                cols: width as usize,
                rows: height as usize,
            })
            .await
            .map_err(|_| anyhow!("ui thread has died"))?;

        Ok(())
    }
}
