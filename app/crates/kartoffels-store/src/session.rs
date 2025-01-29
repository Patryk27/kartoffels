use ahash::AHashMap;
use derivative::Derivative;
use kartoffels_utils::Id;
use rand::distributions::Standard;
use rand::prelude::Distribution;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::hash_map;
use std::fmt;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use tracing::{debug, info};

#[derive(Debug)]
pub struct Session {
    id: SessionId,
    entry: Arc<Mutex<SessionEntry>>,
    entries: Arc<Mutex<AHashMap<SessionId, Arc<Mutex<SessionEntry>>>>>,
}

impl Session {
    pub(crate) fn create(
        entries: Arc<Mutex<AHashMap<SessionId, Arc<Mutex<SessionEntry>>>>>,
    ) -> Self {
        let mut rng = rand::thread_rng();

        loop {
            let id = rng.gen();

            let entry = if let hash_map::Entry::Vacant(entry) =
                entries.lock().unwrap().entry(id)
            {
                info!(?id, "session created");

                entry
                    .insert(Arc::new(Mutex::new(SessionEntry::default())))
                    .clone()
            } else {
                continue;
            };

            break Self { id, entry, entries };
        }
    }

    pub fn id(&self) -> SessionId {
        self.id
    }

    pub fn with<T>(&self, f: impl FnOnce(&mut SessionEntry) -> T) -> T {
        f(&mut self.entry.lock().unwrap())
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        debug!(id = ?self.id, "session dropped");

        self.entries.lock().unwrap().remove(&self.id);
    }
}

#[derive(Debug, Default)]
pub struct SessionEntry {
    role: SessionRole,
    upload: Option<oneshot::Sender<Vec<u8>>>,
}

impl SessionEntry {
    pub fn is_admin(&self) -> bool {
        self.role == SessionRole::Admin
    }

    pub fn make_admin(&mut self) {
        self.role = SessionRole::Admin;
    }

    pub fn request_upload(&mut self) -> SessionUploadInterest {
        let (tx, rx) = oneshot::channel();

        self.upload = Some(tx);

        SessionUploadInterest { rx }
    }

    #[allow(clippy::result_unit_err)]
    pub fn complete_upload(&mut self, src: Vec<u8>) -> Result<(), ()> {
        if let Some(tx) = self.upload.take() {
            _ = tx.send(src);

            Ok(())
        } else {
            Err(())
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SessionRole {
    Admin,

    #[default]
    User,
}

#[derive(Debug)]
pub struct SessionUploadInterest {
    rx: oneshot::Receiver<Vec<u8>>,
}

impl SessionUploadInterest {
    pub fn try_recv(&mut self) -> Option<Vec<u8>> {
        self.rx.try_recv().ok()
    }
}

#[derive(
    Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Derivative,
)]
#[derivative(Debug = "transparent")]
pub struct SessionId(Id);

impl Distribution<SessionId> for Standard {
    fn sample<R>(&self, rng: &mut R) -> SessionId
    where
        R: Rng + ?Sized,
    {
        SessionId(rng.gen())
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
