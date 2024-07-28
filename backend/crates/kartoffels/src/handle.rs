mod systems;

pub use self::systems::*;
use crate::{BotId, ClientUpdate, ClientUpdateRx, World, WorldName};
use anyhow::{anyhow, Context, Result};
use derivative::Derivative;
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

    pub async fn join(
        &self,
        id: Option<BotId>,
    ) -> Result<impl Stream<Item = ClientUpdate>> {
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

#[derive(Derivative)]
#[derivative(Debug)]
pub enum Request {
    Join {
        id: Option<BotId>,

        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<ClientUpdateRx>,
    },

    Pause {
        paused: bool,
    },

    Close {
        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<()>,
    },

    UploadBot {
        #[derivative(Debug = "ignore")]
        src: Cow<'static, [u8]>,

        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<Result<BotId>>,
    },

    RestartBot {
        id: BotId,
    },

    DestroyBot {
        id: BotId,
    },

    SetSpawnPoint {
        at: Option<IVec2>,
    },
}
