mod systems;

pub use self::systems::*;
use crate::{
    BotId, Clock, Dir, EventLetter, EventStream, Map, Object, ObjectId,
    Snapshot, SnapshotStream,
};
use anyhow::{anyhow, Context, Result};
use derivative::Derivative;
use glam::IVec2;
use kartoffels_utils::Id;
use std::borrow::Cow;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, oneshot, watch, OwnedSemaphorePermit};

#[derive(Clone, Debug)]
pub struct Handle {
    pub(super) shared: Arc<HandleShared>,
    pub(super) permit: Option<Arc<OwnedSemaphorePermit>>,
}

impl Handle {
    pub(crate) const ERR: &'static str = "world has crashed";

    pub fn id(&self) -> Id {
        self.shared.id
    }

    pub fn name(&self) -> &str {
        &self.shared.name
    }

    pub fn events(&self) -> Result<EventStream> {
        let events = self
            .shared
            .events
            .as_ref()
            .context("world doesn't have events enabled")?;

        Ok(EventStream::new(events))
    }

    pub fn snapshots(&self) -> SnapshotStream {
        SnapshotStream::new(&self.shared.snapshots)
    }

    pub fn version(&self) -> u64 {
        self.shared.snapshots.borrow().version()
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

    pub async fn create_bots(
        &self,
        reqs: impl IntoIterator<Item = CreateBotRequest>,
    ) -> Result<Vec<BotId>> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::CreateBots {
            reqs: reqs.into_iter().collect(),
            tx,
        })
        .await?;

        rx.await.context(Self::ERR)?.into_iter().collect()
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

    pub async fn delete_bot(&self, id: BotId) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::DeleteBot { id, tx }).await?;

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

    pub async fn create_object(
        &self,
        obj: Object,
        pos: impl Into<Option<IVec2>>,
    ) -> Result<ObjectId> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::CreateObject {
            obj,
            pos: pos.into(),
            tx,
        })
        .await?;

        rx.await.context(Self::ERR)
    }

    pub async fn delete_object(&self, id: ObjectId) -> Result<Option<Object>> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::DeleteObject { id, tx }).await?;

        rx.await.context(Self::ERR)
    }

    pub async fn overclock(&self, clock: Clock) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::Overclock { clock, tx }).await?;

        rx.await.context(Self::ERR)
    }

    async fn send(&self, request: Request) -> Result<()> {
        self.shared
            .tx
            .send(request)
            .await
            .map_err(|_| anyhow!("{}", Self::ERR))?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct HandleShared {
    pub tx: RequestTx,
    pub id: Id,
    pub name: String,
    pub events: Option<broadcast::Sender<EventLetter>>,
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

    CreateBots {
        reqs: Vec<CreateBotRequest>,

        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<Vec<Result<BotId>>>,
    },

    KillBot {
        id: BotId,
        reason: String,

        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<()>,
    },

    DeleteBot {
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

    CreateObject {
        obj: Object,
        pos: Option<IVec2>,

        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<ObjectId>,
    },

    DeleteObject {
        id: ObjectId,

        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<Option<Object>>,
    },

    Overclock {
        clock: Clock,

        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<()>,
    },
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct CreateBotRequest {
    #[derivative(Debug = "ignore")]
    pub src: Cow<'static, [u8]>,
    pub pos: Option<IVec2>,
    pub dir: Option<Dir>,
    pub instant: bool,
    pub oneshot: bool,
}

impl CreateBotRequest {
    pub fn new(src: impl Into<Cow<'static, [u8]>>) -> Self {
        Self {
            src: src.into(),
            pos: None,
            dir: None,
            instant: false,
            oneshot: false,
        }
    }

    pub fn at(mut self, pos: impl Into<Option<IVec2>>) -> Self {
        self.pos = pos.into();
        self
    }

    pub fn facing(mut self, dir: impl Into<Option<Dir>>) -> Self {
        self.dir = dir.into();
        self
    }

    pub fn instant(mut self) -> Self {
        self.instant = true;
        self
    }

    pub fn oneshot(mut self) -> Self {
        self.oneshot = true;
        self
    }
}
