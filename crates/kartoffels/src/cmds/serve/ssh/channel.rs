use anyhow::{anyhow, Result};
use futures_util::sink::drain;
use futures_util::SinkExt;
use glam::uvec2;
use kartoffels_store::Store;
use kartoffels_ui::{Term, TermType};
use russh::server::{Handle as SessionHandle, Session};
use russh::ChannelId;
use std::sync::Arc;
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
        stdin_tx: mpsc::Sender<Vec<u8>>,
    },
}

impl AppChannel {
    pub fn new(store: Arc<Store>, shutdown: CancellationToken) -> Self {
        AppChannel::AwaitingPty { store, shutdown }
    }

    pub async fn data(&mut self, data: &[u8]) -> Result<()> {
        let AppChannel::Ready { stdin_tx } = self else {
            return Err(anyhow!("pty hasn't been allocated yet"));
        };

        stdin_tx
            .send(data.to_vec())
            .await
            .map_err(|_| anyhow!("ui thread has died"))?;

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
            Self::create_term(handle.clone(), id, width, height).await?;

        let result = task::spawn(async move {
            kartoffels_game::start(&mut term, &store).await
        });

        task::spawn(async move {
            let result = select! {
                result = result => Some(result),
                _ = shutdown.cancelled() => None,
            };

            _ = handle
                .data(id, Term::leave_cmds().into_bytes().into())
                .await;

            match result {
                Some(Ok(result)) => {
                    info!("ui task finished: {:?}", result);
                }

                Some(Err(err)) => {
                    info!("ui task crashed: {}", err);

                    _ = handle.data(id, Term::crashed_msg().into()).await;
                }

                None => {
                    info!("ui task aborted: shutting down");

                    _ = handle.data(id, Term::shutting_down_msg().into()).await;
                }
            }

            _ = handle.close(id).await;
        });

        *self = AppChannel::Ready { stdin_tx };

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
            .send(vec![0x04, width as u8, height as u8])
            .await
            .map_err(|_| anyhow!("ui thread has died"))?;

        Ok(())
    }

    async fn create_term(
        handle: SessionHandle,
        id: ChannelId,
        width: u32,
        height: u32,
    ) -> Result<(Term, mpsc::Sender<Vec<u8>>)> {
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

        let term =
            Term::new(TermType::Ssh, stdin, stdout, uvec2(width, height))
                .await?;

        Ok((term, stdin_tx))
    }
}
