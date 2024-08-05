mod systems;

pub use self::systems::*;
use crate::{
    BotId, BotInfo, ConnMsg, ConnMsgRx, Event, EventRx, World, WorldName,
};
use anyhow::{anyhow, Context, Result};
use futures_util::Stream;
use glam::IVec2;
use std::borrow::Cow;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;

#[derive(Clone, Debug)]
pub struct Handle {
    pub(super) name: Arc<WorldName>,
    pub(super) mode: &'static str,
    pub(super) theme: &'static str,
    pub(super) tx: RequestTx,
}

impl Handle {
    const ERR_DIED: &'static str = "world actor has died";

    pub(crate) fn new(world: &World, tx: RequestTx) -> Self {
        Self {
            name: world.name.clone(),
            mode: world.mode.ty(),
            theme: world.theme.ty(),
            tx,
        }
    }

    pub fn name(&self) -> &WorldName {
        &self.name
    }

    pub fn mode(&self) -> &'static str {
        self.mode
    }

    pub fn theme(&self) -> &'static str {
        self.theme
    }

    pub async fn listen(&self) -> Result<impl Stream<Item = Event>> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::Listen { tx }).await?;

        let rx = rx.await.context(Self::ERR_DIED)?;

        Ok(ReceiverStream::new(rx))
    }

    pub async fn join(
        &self,
        id: Option<BotId>,
    ) -> Result<impl Stream<Item = ConnMsg>> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::Join { id, tx }).await?;

        let rx = rx.await.context(Self::ERR_DIED)?;

        Ok(ReceiverStream::new(rx))
    }

    pub async fn pause(&self, paused: bool) -> Result<()> {
        self.send(Request::Pause { paused }).await?;

        Ok(())
    }

    pub async fn close(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::Close { tx }).await?;

        rx.await.context(Self::ERR_DIED)
    }

    pub async fn upload_bot(&self, src: Cow<'static, [u8]>) -> Result<BotId> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::UploadBot { src, tx }).await?;

        rx.await.context(Self::ERR_DIED)?
    }

    pub async fn restart_bot(&self, id: BotId) -> Result<()> {
        self.send(Request::RestartBot { id }).await?;

        Ok(())
    }

    pub async fn destroy_bot(&self, id: BotId) -> Result<()> {
        self.send(Request::DestroyBot { id }).await?;

        Ok(())
    }

    pub async fn get_bots(&self) -> Result<Vec<BotInfo>> {
        let (tx, rx) = oneshot::channel();

        self.send(Request::GetBots { tx }).await?;

        rx.await.context(Self::ERR_DIED)
    }

    pub async fn set_spawn_point(&self, at: Option<IVec2>) -> Result<()> {
        self.send(Request::SetSpawnPoint { at }).await?;

        Ok(())
    }

    async fn send(&self, request: Request) -> Result<()> {
        self.tx
            .send(request)
            .await
            .map_err(|_| anyhow!("{}", Self::ERR_DIED))?;

        Ok(())
    }
}

pub type RequestTx = mpsc::Sender<Request>;
pub type RequestRx = mpsc::Receiver<Request>;

pub enum Request {
    Listen {
        tx: oneshot::Sender<EventRx>,
    },

    Join {
        id: Option<BotId>,
        tx: oneshot::Sender<ConnMsgRx>,
    },

    Pause {
        paused: bool,
    },

    Close {
        tx: oneshot::Sender<()>,
    },

    UploadBot {
        src: Cow<'static, [u8]>,
        tx: oneshot::Sender<Result<BotId>>,
    },

    RestartBot {
        id: BotId,
    },

    DestroyBot {
        id: BotId,
    },

    GetBots {
        tx: oneshot::Sender<Vec<BotInfo>>,
    },

    SetSpawnPoint {
        at: Option<IVec2>,
    },
}
