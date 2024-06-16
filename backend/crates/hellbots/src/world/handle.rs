use super::WorldRequest;
use crate::{BotId, WorldName, WorldUpdate};
use anyhow::{anyhow, Context, Result};
use futures_util::Stream;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;

#[derive(Clone, Debug)]
pub struct WorldHandle {
    pub(super) name: Arc<WorldName>,
    pub(super) mode: &'static str,
    pub(super) theme: &'static str,
    pub(super) tx: mpsc::Sender<WorldRequest>,
}

impl WorldHandle {
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

    pub async fn create_bot(&self, src: Vec<u8>) -> Result<BotId> {
        let (tx, rx) = oneshot::channel();

        self.tx
            .send(WorldRequest::CreateBot { src, tx })
            .await
            .map_err(|_| anyhow!("{}", Self::ERR_DIED))?;

        rx.await.context(Self::ERR_DIED)?
    }

    pub async fn join(
        &self,
        id: Option<BotId>,
    ) -> Result<impl Stream<Item = WorldUpdate>> {
        let (tx, rx) = oneshot::channel();

        self.tx
            .send(WorldRequest::Join { id, tx })
            .await
            .map_err(|_| anyhow!("{}", Self::ERR_DIED))?;

        let rx = rx.await.context(Self::ERR_DIED)?;

        Ok(ReceiverStream::new(rx))
    }
}
