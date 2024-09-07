mod systems;

pub use self::systems::*;
use crate::{BotId, Event, Snapshot};
use anyhow::{anyhow, Context, Result};
use futures_util::Stream;
use glam::IVec2;
use kartoffels_utils::Id;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, oneshot, watch};
use tokio_stream::wrappers::{BroadcastStream, WatchStream};
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
        BroadcastStream::new(self.inner.events.subscribe())
            .filter_map(|msg| msg.ok())
    }

    pub fn snapshots(&self) -> SnapshotStream {
        WatchStream::new(self.inner.snapshots.subscribe())
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

    pub async fn set_spawn_point(&self, at: IVec2) -> Result<()> {
        self.send(Request::SetSpawnPoint { at }).await?;

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
    pub snapshots: watch::Sender<Arc<Snapshot>>,
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

    SetSpawnPoint {
        at: IVec2,
    },
}

// ---

pub type EventStream = impl Stream<Item = Arc<Event>> + Send + Sync + Unpin;

pub trait EventStreamExt {
    fn next_or_err(&mut self) -> impl Future<Output = Result<Arc<Event>>>;
}

impl<T> EventStreamExt for T
where
    T: Stream<Item = Arc<Event>> + Unpin,
{
    async fn next_or_err(&mut self) -> Result<Arc<Event>> {
        self.next().await.context(Handle::ERR)
    }
}

// ---

pub type SnapshotStream =
    impl Stream<Item = Arc<Snapshot>> + Send + Sync + Unpin;

pub trait SnapshotStreamExt {
    fn next_or_err(&mut self) -> impl Future<Output = Result<Arc<Snapshot>>>;
}

impl<T> SnapshotStreamExt for T
where
    T: Stream<Item = Arc<Snapshot>> + Unpin,
{
    async fn next_or_err(&mut self) -> Result<Arc<Snapshot>> {
        self.next().await.context(Handle::ERR)
    }
}
