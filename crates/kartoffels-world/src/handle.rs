mod systems;

pub use self::systems::*;
use crate::{BotId, Event, Snapshot};
use anyhow::{anyhow, Context, Result};
use futures_util::Stream;
use glam::IVec2;
use kartoffels_utils::Id;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

#[derive(Clone, Debug)]
pub struct Handle {
    pub(super) inner: Arc<HandleInner>,
}

impl Handle {
    const ERR: &'static str = "lost connection to the world";

    pub fn id(&self) -> Id {
        self.inner.id
    }

    pub fn name(&self) -> &str {
        &self.inner.name
    }

    pub fn events(&self) -> EventStream {
        let stream = BroadcastStream::new(self.inner.events.subscribe())
            .filter_map(|msg| msg.ok());

        EventStream {
            id: self.inner.id,
            stream: Box::new(stream),
        }
    }

    pub fn snapshots(&self) -> SnapshotStream {
        let stream = BroadcastStream::new(self.inner.snapshots.subscribe())
            .filter_map(|msg| msg.ok());

        SnapshotStream {
            id: self.inner.id,
            stream: Box::new(stream),
        }
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
        self.inner
            .tx
            .send(request)
            .await
            .map_err(|_| anyhow!("{}", Self::ERR))?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct HandleInner {
    pub tx: RequestTx,
    pub id: Id,
    pub name: Arc<String>,
    pub events: broadcast::Sender<Arc<Event>>,
    pub snapshots: broadcast::Sender<Arc<Snapshot>>,
}

pub type RequestTx = mpsc::Sender<Request>;
pub type RequestRx = mpsc::Receiver<Request>;

pub enum Request {
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

pub struct EventStream {
    id: Id,
    stream: Box<dyn Stream<Item = Arc<Event>> + Send + Sync + Unpin>,
}

impl EventStream {
    pub async fn next(&mut self) -> Result<Arc<Event>> {
        self.stream
            .next()
            .await
            .with_context(|| format!("lost connection to world `{}`", self.id))
    }
}

pub struct SnapshotStream {
    id: Id,
    stream: Box<dyn Stream<Item = Arc<Snapshot>> + Send + Sync + Unpin>,
}

impl SnapshotStream {
    pub async fn next(&mut self) -> Result<Arc<Snapshot>> {
        self.stream
            .next()
            .await
            .with_context(|| format!("lost connection to world `{}`", self.id))
    }

    pub fn into_inner(
        self,
    ) -> Box<dyn Stream<Item = Arc<Snapshot>> + Send + Sync + Unpin> {
        self.stream
    }
}
