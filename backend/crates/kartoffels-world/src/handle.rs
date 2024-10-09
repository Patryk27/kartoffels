mod systems;

pub use self::systems::*;
use crate::{BotId, ClockSpeed, Dir, Event, Map, Snapshot};
use anyhow::{anyhow, Context, Result};
use derivative::Derivative;
use futures_util::Stream;
use glam::IVec2;
use kartoffels_utils::Id;
use std::borrow::Cow;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, oneshot, watch, OwnedSemaphorePermit};
use tokio_stream::wrappers::{BroadcastStream, WatchStream};
use tokio_stream::StreamExt;

#[derive(Clone, Debug)]
pub struct Handle {
    pub(super) inner: Arc<HandleInner>,
    pub(super) permit: Option<Arc<OwnedSemaphorePermit>>,
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

    pub fn with_permit(mut self, permit: OwnedSemaphorePermit) -> Self {
        self.permit = Some(Arc::new(permit));
        self
    }

    pub async fn tick(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::Tick { tx }).await?;

        rx.await.context(Self::ERR)
    }

    pub async fn pause(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::Pause { tx }).await?;

        rx.await.context(Self::ERR)
    }

    pub async fn resume(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::Resume { tx }).await?;

        rx.await.context(Self::ERR)
    }

    pub async fn shutdown(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::Shutdown { tx }).await?;

        rx.await.context(Self::ERR)
    }

    pub async fn create_bot(&self, req: CreateBotRequest) -> Result<BotId> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::CreateBot { req, tx }).await?;

        rx.await.context(Self::ERR)?
    }

    pub async fn kill_bot(
        &self,
        id: BotId,
        reason: impl ToString,
    ) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::KillBot {
            id,
            reason: reason.to_string(),
            tx,
        })
        .await?;

        rx.await.context(Self::ERR)
    }

    pub async fn destroy_bot(&self, id: BotId) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::DestroyBot { id, tx }).await?;

        rx.await.context(Self::ERR)
    }

    pub async fn set_map(&self, map: Map) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::SetMap { map, tx }).await?;

        rx.await.context(Self::ERR)
    }

    pub async fn set_spawn(
        &self,
        point: impl Into<Option<IVec2>>,
        dir: impl Into<Option<Dir>>,
    ) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::SetSpawn {
            point: point.into(),
            dir: dir.into(),
            tx,
        })
        .await?;

        rx.await.context(Self::ERR)
    }

    pub async fn overclock(&self, speed: ClockSpeed) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::Overclock { speed, tx }).await?;

        rx.await.context(Self::ERR)?
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

#[derive(Derivative)]
#[derivative(Debug)]
pub enum Request {
    Tick {
        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<()>,
    },

    Pause {
        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<()>,
    },

    Resume {
        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<()>,
    },

    Shutdown {
        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<()>,
    },

    CreateBot {
        req: CreateBotRequest,

        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<Result<BotId>>,
    },

    KillBot {
        id: BotId,
        reason: String,

        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<()>,
    },

    DestroyBot {
        id: BotId,

        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<()>,
    },

    SetMap {
        map: Map,

        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<()>,
    },

    SetSpawn {
        point: Option<IVec2>,
        dir: Option<Dir>,

        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<()>,
    },

    Overclock {
        speed: ClockSpeed,

        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<Result<()>>,
    },
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct CreateBotRequest {
    #[derivative(Debug = "ignore")]
    pub src: Cow<'static, [u8]>,
    pub pos: Option<IVec2>,
    pub dir: Option<Dir>,
    pub oneshot: bool,
}

impl CreateBotRequest {
    pub fn new(src: impl Into<Cow<'static, [u8]>>) -> Self {
        Self {
            src: src.into(),
            pos: None,
            dir: None,
            oneshot: false,
        }
    }

    pub fn at(mut self, pos: IVec2) -> Self {
        self.pos = Some(pos);
        self
    }

    pub fn facing(mut self, dir: Dir) -> Self {
        self.dir = Some(dir);
        self
    }

    pub fn oneshot(mut self) -> Self {
        self.oneshot = true;
        self
    }
}

// ---

pub type EventStream = impl Stream<Item = Arc<Event>> + Send + Unpin;

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

pub type SnapshotStream = impl Stream<Item = Arc<Snapshot>> + Send + Unpin;

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
