use crate::{BotId, Update, UpdateRx, WorldName};
use anyhow::{anyhow, Context, Result};
use derivative::Derivative;
use futures_util::Stream;
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

    pub fn name(&self) -> &WorldName {
        &self.name
    }

    pub fn mode(&self) -> &'static str {
        self.mode
    }

    pub fn theme(&self) -> &'static str {
        self.theme
    }

    pub async fn upload(&self, src: Vec<u8>) -> Result<BotId> {
        let (tx, rx) = oneshot::channel();

        self.tx
            .send(Request::Upload { src, tx })
            .await
            .map_err(|_| anyhow!("{}", Self::ERR_DIED))?;

        rx.await.context(Self::ERR_DIED)?
    }

    pub async fn join(
        &self,
        id: Option<BotId>,
    ) -> Result<impl Stream<Item = Update>> {
        let (tx, rx) = oneshot::channel();

        self.tx
            .send(Request::Join { id, tx })
            .await
            .map_err(|_| anyhow!("{}", Self::ERR_DIED))?;

        let rx = rx.await.context(Self::ERR_DIED)?;

        Ok(ReceiverStream::new(rx))
    }
}

pub type RequestTx = mpsc::Sender<Request>;
pub type RequestRx = mpsc::Receiver<Request>;

#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) enum Request {
    Upload {
        #[derivative(Debug = "ignore")]
        src: Vec<u8>,

        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<Result<BotId>>,
    },

    Join {
        id: Option<BotId>,

        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<UpdateRx>,
    },
}
