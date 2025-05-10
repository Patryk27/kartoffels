use crate::common;
use anyhow::{anyhow, Result};
use glam::UVec2;
use kartoffels_front::{Frame, FrameType, StdinEvent};
use kartoffels_store::Store;
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
        stdin: mpsc::Sender<StdinEvent>,
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
            .send(StdinEvent::Input(data.to_vec().into()))
            .await
            .map_err(|_| anyhow!("lost the frame"))?;

        Ok(())
    }

    pub async fn pty_request(
        &mut self,
        id: ChannelId,
        size: UVec2,
        session: &mut Session,
    ) -> Result<()> {
        let AppChannelState::AwaitingPty { store, shutdown } = &mut self.state
        else {
            return Err(anyhow!("pty has been already allocated"));
        };

        let store = store.clone();
        let shutdown = shutdown.clone();
        let handle = session.handle();

        let (term, stdin) =
            Self::create_term(handle.clone(), id, size, self.span.clone())?;

        task::spawn(
            common::start_session(store, term, shutdown)
                .instrument(self.span.clone()),
        );

        self.state = AppChannelState::Ready { stdin };

        Ok(())
    }

    pub async fn window_change_request(&mut self, size: UVec2) -> Result<()> {
        let AppChannelState::Ready { stdin } = &mut self.state else {
            return Err(anyhow!("pty hasn't been allocated yet"));
        };

        stdin
            .send(StdinEvent::Resized(size))
            .await
            .map_err(|_| anyhow!("lost the frame"))?;

        Ok(())
    }

    fn create_term(
        handle: SessionHandle,
        id: ChannelId,
        size: UVec2,
        span: Span,
    ) -> Result<(Frame, mpsc::Sender<StdinEvent>)> {
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

        let frame = Frame::new(FrameType::Ssh, size, stdin_rx, stdout)?;

        Ok((frame, stdin_tx))
    }
}

impl Drop for AppChannel {
    fn drop(&mut self) {
        info!(parent: &self.span, "channel closed");
    }
}
