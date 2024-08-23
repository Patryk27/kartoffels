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
    pub(super) tx: RequestTx,
}

impl Handle {
    const ERR_DIED: &'static str = "world actor has died";

    pub(crate) fn new(world: &World, tx: RequestTx) -> Self {
        Self {
            name: world.name.clone(),
            tx,
        }
    }

    pub fn name(&self) -> &WorldName {
        &self.name
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

    pub async fn create_bot(
        &self,
        src: Cow<'static, [u8]>,
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

    CreateBot {
        src: Cow<'static, [u8]>,
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

    GetBots {
        tx: oneshot::Sender<Vec<BotInfo>>,
    },
}
