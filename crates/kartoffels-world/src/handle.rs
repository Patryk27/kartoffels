mod systems;

pub use self::systems::*;
use crate::{BotId, Snapshot};
use anyhow::{anyhow, Context, Result};
use futures_util::Stream;
use glam::IVec2;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

#[derive(Clone, Debug)]
pub struct Handle {
    pub(super) tx: RequestTx,
    pub(super) name: Arc<String>,
}

impl Handle {
    const ERR: &'static str = "world has crashed";

    pub(crate) fn new(tx: RequestTx, name: Arc<String>) -> Self {
        Self { tx, name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn listen(&self) -> Result<impl Stream<Item = Arc<Snapshot>>> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::Listen { tx }).await?;

        let rx = rx.await.context(Self::ERR)?;

        Ok(BroadcastStream::new(rx).filter_map(|msg| msg.ok()))
    }

    pub async fn pause(&self, paused: bool) -> Result<()> {
        self.send(Request::Pause { paused }).await?;

        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::Shutdown { tx }).await?;

        rx.await.context(Self::ERR)
    }

    pub async fn create_bot(
        &self,
        src: Vec<u8>,
        pos: Option<IVec2>,
        ephemeral: bool,
    ) -> Result<BotId> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::CreateBot {
            src,
            pos,
            ephemeral,
            tx,
        })
        .await?;

        rx.await.context(Self::ERR)?
    }

    pub async fn restart_bot(&self, id: BotId) -> Result<()> {
        self.send(Request::RestartBot { id }).await?;

        Ok(())
    }

    pub async fn destroy_bot(&self, id: BotId) -> Result<()> {
        self.send(Request::DestroyBot { id }).await?;

        Ok(())
    }

    async fn send(&self, request: Request) -> Result<()> {
        self.tx
            .send(request)
            .await
            .map_err(|_| anyhow!("{}", Self::ERR))?;

        Ok(())
    }
}

pub type RequestTx = mpsc::Sender<Request>;
pub type RequestRx = mpsc::Receiver<Request>;

pub enum Request {
    Listen {
        tx: oneshot::Sender<broadcast::Receiver<Arc<Snapshot>>>,
    },

    Pause {
        paused: bool,
    },

    Shutdown {
        tx: oneshot::Sender<()>,
    },

    CreateBot {
        src: Vec<u8>,
        pos: Option<IVec2>,
        ephemeral: bool,
        tx: oneshot::Sender<Result<BotId>>,
    },

    RestartBot {
        id: BotId,
    },

    DestroyBot {
        id: BotId,
    },
}
