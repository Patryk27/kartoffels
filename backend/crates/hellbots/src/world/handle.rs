use super::WorldMsg;
use crate::{BotId, WorldName, WorldSnapshot};
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, RwLock};

#[derive(Clone, Debug)]
pub struct WorldHandle {
    pub(super) name: Arc<WorldName>,
    pub(super) mode: &'static str,
    pub(super) theme: &'static str,
    pub(super) tx: mpsc::UnboundedSender<WorldMsg>,
    pub(super) snapshot: Arc<RwLock<WorldSnapshot>>,
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

    pub fn snapshot(&self) -> &RwLock<WorldSnapshot> {
        &self.snapshot
    }

    pub async fn create_bot(&self, src: Vec<u8>) -> Result<BotId> {
        let (tx, rx) = oneshot::channel();

        self.tx
            .send(WorldMsg::CreateBot { src, tx })
            .context(Self::ERR_DIED)?;

        rx.await.context(Self::ERR_DIED)?
    }

    pub async fn update_bot(&self, id: BotId, src: Vec<u8>) -> Result<()> {
        let (tx, rx) = oneshot::channel();

        self.tx
            .send(WorldMsg::UpdateBot { id, src, tx })
            .context(Self::ERR_DIED)?;

        rx.await.context(Self::ERR_DIED)?
    }
}
