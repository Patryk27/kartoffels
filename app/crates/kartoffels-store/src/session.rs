mod entry;
mod id;
mod role;

pub use self::entry::*;
pub use self::id::*;
pub use self::role::*;
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task;

#[derive(Clone, Debug)]
pub struct Session {
    id: SessionId,
    entry: Arc<Mutex<SessionEntry>>,
    on_abandoned: Option<Arc<mpsc::Sender<SessionId>>>,
}

impl Session {
    pub(crate) fn new(id: SessionId, entry: Arc<Mutex<SessionEntry>>) -> Self {
        Self {
            id,
            entry,
            on_abandoned: None,
        }
    }

    pub fn id(&self) -> SessionId {
        self.id
    }

    pub fn with<T>(&self, f: impl FnOnce(&mut SessionEntry) -> T) -> T {
        f(&mut self.entry.lock())
    }

    pub(crate) fn on_abandoned(mut self, tx: mpsc::Sender<SessionId>) -> Self {
        self.on_abandoned = Some(Arc::new(tx));
        self
    }
}

impl Drop for Session {
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
