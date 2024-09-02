use anyhow::{anyhow, Result};
use futures_util::sink::drain;
use futures_util::SinkExt;
use glam::uvec2;
use itertools::Either;
use kartoffels_store::Store;
use kartoffels_ui::Term;
use russh::server::{Handle as SessionHandle, Session};
use russh::ChannelId;
use std::sync::Arc;
use termwiz::input::{InputEvent, InputParser, KeyCode, Modifiers};
use tokio::sync::mpsc;
use tokio::{select, task};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;
use tracing::info;

#[derive(Debug)]
pub enum AppChannel {
    AwaitingPty {
        store: Arc<Store>,
        shutdown: CancellationToken,
    },

    Ready {
        stdin_tx: mpsc::Sender<InputEvent>,
        stdin_parser: InputParser,
    },
}

impl AppChannel {
    pub fn new(store: Arc<Store>, shutdown: CancellationToken) -> Self {
        AppChannel::AwaitingPty { store, shutdown }
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
                    session
                        .data(id, Term::leave_sequence().into_bytes().into());

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
        let AppChannel::AwaitingPty { store, shutdown } = self else {
            return Err(anyhow!("pty has been already allocated"));
        };

        let store = store.clone();
        let shutdown = shutdown.clone();
        let handle = session.handle();

        let (mut term, stdin_tx) =
            Self::create_term(handle.clone(), id, width, height)?;

        let ui =
            task::spawn(
                async move { kartoffels_ui::main(&mut term, &store).await },
            );

        task::spawn(async move {
            let result = select! {
                result = ui => Either::Left(result),
                _ = shutdown.cancelled() => Either::Right(()),
            };

            _ = handle
                .data(id, Term::leave_sequence().into_bytes().into())
                .await;

            match result {
                Either::Left(Ok(result)) => {
                    info!("ui task returned: {:?}", result);
                }

                Either::Left(Err(err)) => {
                    info!("ui task crashed: {}", err);

                    _ = handle
                        .data(
                            id,
                            "whoopsie, the game has crashed!\r\n"
                                .to_string()
                                .into_bytes()
                                .into(),
                        )
                        .await;
                }

                Either::Right(_) => {
                    info!("ui task aborted: shutting down");

                    _ = handle
                        .data(
                            id,
                            "whoopsie, the server is shutting down!\r\n"
                                .to_string()
                                .into_bytes()
                                .into(),
                        )
                        .await;
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

    fn create_term(
        handle: SessionHandle,
        id: ChannelId,
        width: u32,
        height: u32,
    ) -> Result<(Term, mpsc::Sender<InputEvent>)> {
        let (stdin_tx, stdin_rx) = mpsc::channel(32);
        let stdin = Box::pin(ReceiverStream::new(stdin_rx).map(Ok));

        let stdout = Box::pin({
            let handle = handle.clone();

            drain().with(move |stdout: Vec<u8>| {
                let handle = handle.clone();

                async move {
                    match handle.data(id, stdout.into()).await {
                        Ok(_) => Ok(()),
                        Err(_) => Err(anyhow!("ssh channel died")),
                    }
                }
            })
        });

        let term = Term::new(stdin, stdout, uvec2(width, height))?;

        Ok((term, stdin_tx))
    }
}
