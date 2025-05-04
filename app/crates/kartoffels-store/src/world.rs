mod entry;
mod id;
mod vis;

pub use self::entry::*;
pub use self::id::*;
pub use self::vis::*;
use kartoffels_world::prelude::Handle as WorldHandle;
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task;

#[derive(Clone, Debug)]
pub struct World {
    id: WorldId,
    entry: Arc<WorldEntry>,
    on_abandoned: Option<Arc<mpsc::Sender<WorldId>>>,
}

impl World {
    pub(crate) fn new(id: WorldId, entry: Arc<WorldEntry>) -> Self {
        Self {
            id,
            entry,
            on_abandoned: None,
        }
    }

    pub fn id(&self) -> WorldId {
        self.id
    }

    pub fn vis(&self) -> WorldVis {
        self.entry.vis
    }

    pub fn path(&self) -> Option<&Path> {
        self.entry.path.as_deref()
    }

    pub(crate) fn on_abandoned(mut self, tx: mpsc::Sender<WorldId>) -> Self {
        self.on_abandoned = Some(Arc::new(tx));
        self
    }
}

impl Deref for World {
    type Target = WorldHandle;

    fn deref(&self) -> &Self::Target {
        &self.entry.handle
    }
}

impl Drop for World {
    fn drop(&mut self) {
        if let Some(tx) = self.on_abandoned.take()
            && let Ok(tx) = Arc::try_unwrap(tx)
        {
            let id = self.id;

            task::spawn(async move {
                _ = tx.send(id).await;
            });
        }
    }
}
