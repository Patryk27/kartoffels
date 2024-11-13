mod systems;

pub use self::systems::*;
use crate::{BotId, ClockSpeed, Dir, Map, Object, Snapshot};
use ahash::HashSet;
use anyhow::{anyhow, Context, Result};
use derivative::Derivative;
use glam::IVec2;
use kartoffels_utils::Id;
use std::borrow::Cow;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, watch, OwnedSemaphorePermit};
use tokio_stream::wrappers::WatchStream;
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

    pub fn snapshots(&self) -> SnapshotStream {
        SnapshotStream {
            rx: WatchStream::new(self.inner.snapshots.subscribe()),
        }
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

    pub async fn put_object(
        &self,
        pos: IVec2,
        obj: impl Into<Object>,
    ) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::PutObject {
            pos,
            obj: obj.into(),
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

    PutObject {
        pos: IVec2,
        obj: Object,

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

    pub fn spawn_at_once(mut self) -> Self {
        self.instant = true;
        self
    }

    pub fn oneshot(mut self) -> Self {
        self.oneshot = true;
        self
    }
}

#[derive(Debug)]
pub struct SnapshotStream {
    rx: WatchStream<Arc<Snapshot>>,
}

impl SnapshotStream {
    pub async fn next(&mut self) -> Result<Arc<Snapshot>> {
        self.rx.next().await.with_context(|| Handle::ERR)
    }

    pub async fn wait_for_bot(&mut self, id: BotId) -> Result<()> {
        loop {
            if self.next().await?.bots().alive().has(id) {
                return Ok(());
            }
        }
    }

    pub async fn wait_until_bot_is_spawned(&mut self) -> Result<BotId> {
        let known_bots: HashSet<_> = self
            .next()
            .await?
            .bots()
            .alive()
            .iter()
            .map(|bot| bot.id)
            .collect();

        loop {
            let curr_bots: HashSet<_> = self
                .next()
                .await?
                .bots()
                .alive()
                .iter()
                .map(|bot| bot.id)
                .collect();

            if let Some(id) = curr_bots.difference(&known_bots).next() {
                return Ok(*id);
            }
        }
    }

    pub async fn wait_until_bot_is_killed(&mut self) -> Result<()> {
        let known_bots = self.next().await?.bots().alive().len();

        loop {
            if self.next().await?.bots().alive().len() < known_bots {
                return Ok(());
            }
        }
    }
}
