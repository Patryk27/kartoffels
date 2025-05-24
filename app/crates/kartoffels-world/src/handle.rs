mod systems;

pub use self::systems::*;
use crate::{
    AbsDir, BotId, Clock, EventEnvelope, EventStream, Map, Object, ObjectId,
    Policy, Snapshot, SnapshotStream, WorldBuffer,
};
use anyhow::{anyhow, Context, Result};
use arc_swap::{ArcSwap, Guard};
use derivative::Derivative;
use glam::IVec2;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, oneshot, watch};
use tracing::Span;

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct Handle {
    shared: Arc<SharedHandle>,
}

impl Handle {
    pub(crate) const ERR: &'static str = "world has crashed";

    pub(crate) fn new(shared: SharedHandle) -> Self {
        Self {
            shared: Arc::new(shared),
        }
    }

    pub fn name(&self) -> Guard<Arc<String>> {
        self.shared.name.load()
    }

    pub fn rename(&self, name: String) {
        self.shared.name.store(Arc::new(name));
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
        self.shared.snapshots.borrow().version
    }

    pub async fn ping(&self) -> Result<()> {
        self.send(|tx| Request::Ping { tx }).await
    }

    pub async fn tick(&self, fuel: u32) -> Result<()> {
        self.send(|tx| Request::Tick { fuel, tx }).await
    }

    pub async fn pause(&self) -> Result<()> {
        self.send(|tx| Request::Pause { tx }).await
    }

    pub async fn resume(&self) -> Result<()> {
        self.send(|tx| Request::Resume { tx }).await
    }

    pub async fn shutdown(&self) -> Result<WorldBuffer> {
        self.send(|tx| Request::Shutdown { tx }).await
    }

    pub async fn save(&self) -> Result<WorldBuffer> {
        self.send(|tx| Request::Save { tx }).await
    }

    pub async fn get_policy(&self) -> Result<Policy> {
        self.send(|tx| Request::GetPolicy { tx }).await
    }

    pub async fn set_policy(&self, policy: Policy) -> Result<()> {
        self.send(|tx| Request::SetPolicy { policy, tx }).await
    }

    pub async fn create_bot(&self, req: CreateBotRequest) -> Result<BotId> {
        self.send(|tx| Request::CreateBot { req, tx }).await?
    }

    pub async fn kill_bot(
        &self,
        id: BotId,
        reason: impl ToString,
    ) -> Result<()> {
        self.send(|tx| Request::KillBot {
            id,
            reason: reason.to_string(),
            tx,
        })
        .await
    }

    pub async fn delete_bot(&self, id: BotId) -> Result<()> {
        self.send(|tx| Request::DeleteBot { id, tx }).await
    }

    pub async fn set_map(&self, map: Map) -> Result<()> {
        self.send(|tx| Request::SetMap { map, tx }).await
    }

    pub async fn set_spawn(
        &self,
        pos: impl Into<Option<IVec2>>,
        dir: impl Into<Option<AbsDir>>,
    ) -> Result<()> {
        self.send(|tx| Request::SetSpawn {
            pos: pos.into(),
            dir: dir.into(),
            tx,
        })
        .await
    }

    pub async fn create_object(
        &self,
        obj: Object,
        pos: impl Into<Option<IVec2>>,
    ) -> Result<ObjectId> {
        self.send(|tx| Request::CreateObject {
            obj,
            pos: pos.into(),
            tx,
        })
        .await
    }

    pub async fn delete_object(&self, id: ObjectId) -> Result<Option<Object>> {
        self.send(|tx| Request::DeleteObject { id, tx }).await
    }

    pub async fn overclock(&self, clock: Clock) -> Result<()> {
        self.send(|tx| Request::Overclock { clock, tx }).await
    }

    async fn send<T>(
        &self,
        req: impl FnOnce(oneshot::Sender<T>) -> Request,
    ) -> Result<T> {
        let (tx, rx) = oneshot::channel();

        self.send_ex(req(tx)).await?;

        rx.await.context(Self::ERR)
    }

    async fn send_ex(&self, req: Request) -> Result<()> {
        self.shared
            .tx
            .send((Span::current(), req))
            .await
            .map_err(|_| anyhow!("{}", Self::ERR))?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct SharedHandle {
    pub tx: mpsc::Sender<(Span, Request)>,
    pub name: Arc<ArcSwap<String>>,
    pub events: Option<broadcast::Sender<EventEnvelope>>,
    pub snapshots: watch::Sender<Arc<Snapshot>>,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub enum Request {
    Ping {
        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<()>,
    },

    Tick {
        fuel: u32,
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
        tx: oneshot::Sender<WorldBuffer>,
    },

    Save {
        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<WorldBuffer>,
    },

    GetPolicy {
        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<Policy>,
    },

    SetPolicy {
        policy: Policy,
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
        pos: Option<IVec2>,
        dir: Option<AbsDir>,
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
    pub src: Vec<u8>,
    pub pos: Option<IVec2>,
    pub dir: Option<AbsDir>,
    pub instant: bool,
    pub oneshot: bool,
}

impl CreateBotRequest {
    pub fn new(src: impl Into<Vec<u8>>) -> Self {
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

    pub fn facing(mut self, dir: impl Into<Option<AbsDir>>) -> Self {
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
