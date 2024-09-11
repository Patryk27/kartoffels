use crate::common;
use anyhow::{anyhow, Result};
use glam::uvec2;
use kartoffels_store::Store;
use kartoffels_ui::{Term, TermType};
use russh::server::{Handle as SessionHandle, Session};
use russh::ChannelId;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task;
use tokio_util::sync::CancellationToken;
use tracing::{info, info_span, Instrument, Span};

#[derive(Debug)]
pub struct AppChannel {
    state: AppChannelState,
    span: Span,
}

#[derive(Debug)]
enum AppChannelState {
    AwaitingPty {
        store: Arc<Store>,
        shutdown: CancellationToken,
    },

    Ready {
        stdin: mpsc::Sender<Vec<u8>>,
    },
}

impl AppChannel {
    pub fn new(
        id: ChannelId,
        store: Arc<Store>,
        shutdown: CancellationToken,
        span: &Span,
    ) -> Self {
        let state = AppChannelState::AwaitingPty { store, shutdown };
        let span = info_span!(parent: span, "chan", %id);

        info!(parent: &span, "channel opened");

        Self { state, span }
    }

    pub async fn data(&mut self, data: &[u8]) -> Result<()> {
        let AppChannelState::Ready { stdin: stdin_tx } = &mut self.state else {
            return Err(anyhow!("pty hasn't been allocated yet"));
        };

        stdin_tx
            .send(data.to_vec())
            .await
            .map_err(|_| anyhow!("lost ui"))?;

        Ok(())
    }

    pub async fn pty_request(
        &mut self,
        id: ChannelId,
        width: u32,
        height: u32,
        session: &mut Session,
    ) -> Result<()> {
        let AppChannelState::AwaitingPty { store, shutdown } = &mut self.state
        else {
            return Err(anyhow!("pty has been already allocated"));
        };

        let store = store.clone();
        let shutdown = shutdown.clone();
        let handle = session.handle();

        let (term, stdin) = Self::create_term(
            handle.clone(),
            id,
            width,
            height,
            self.span.clone(),
        )?;

        task::spawn(
            async move {
                common::start_session(term, store, shutdown).await;
            }
            .instrument(self.span.clone()),
        );

        self.state = AppChannelState::Ready { stdin };

        Ok(())
    }

    pub async fn window_change_request(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<()> {
        let AppChannelState::Ready { stdin } = &mut self.state else {
            return Err(anyhow!("pty hasn't been allocated yet"));
        };

        let width = width.min(255);
        let height = height.min(255);

        stdin
            .send(vec![Term::CMD_RESIZE, width as u8, height as u8])
            .await
            .map_err(|_| anyhow!("lost ui"))?;

        Ok(())
    }

    fn create_term(
        handle: SessionHandle,
        id: ChannelId,
        width: u32,
        height: u32,
        span: Span,
    ) -> Result<(Term, mpsc::Sender<Vec<u8>>)> {
        let (stdin_tx, stdin_rx) = mpsc::channel(1);

        let stdout = {
            let (tx, mut rx) = mpsc::channel::<Vec<u8>>(1);

            task::spawn(
                async move {
                    while let Some(msg) = rx.recv().await {
                        if handle.data(id, msg.into()).await.is_err() {
                            info!(
                                "couldn't push data into socket, killing \
                                 stdout",
                            );
                        }
                    }

                    _ = handle.close(id).await;
                }
                .instrument(span),
            );

            tx
        };

        let size = uvec2(width, height);
        let term = Term::new(TermType::Ssh, stdin_rx, stdout, size)?;

        Ok((term, stdin_tx))
    }
}

impl Drop for AppChannel {
    fn drop(&mut self) {
        info!(parent: &self.span, "channel closed");
    }
}
