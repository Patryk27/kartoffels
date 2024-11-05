#![feature(try_blocks)]

mod open;

use ahash::AHashMap;
use anyhow::{anyhow, Result};
use kartoffels_utils::Id;
use kartoffels_world::prelude::{Config as WorldConfig, Handle as WorldHandle};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::hash_map;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::{oneshot, Semaphore};
use tracing::{debug, info};

#[derive(Debug)]
pub struct Store {
    dir: Option<PathBuf>,
    public_worlds: Vec<WorldHandle>,
    private_worlds: Mutex<Vec<WorldHandle>>,
    private_worlds_sem: Arc<Semaphore>,
    sessions: Arc<Mutex<AHashMap<SessionId, Session>>>,
    testing: bool,
}

impl Store {
    pub async fn open(dir: Option<&Path>, bench: bool) -> Result<Self> {
        info!("opening");

        let public_worlds = open::load_worlds(dir, bench).await?;

        info!("ready");

        Ok(Self {
            dir: dir.map(|dir| dir.to_owned()),
            public_worlds,
            private_worlds: Default::default(),
            private_worlds_sem: Arc::new(Semaphore::new(128)), // TODO make configurable
            sessions: Default::default(),
            testing: false,
        })
    }

    pub async fn test(worlds: impl IntoIterator<Item = WorldHandle>) -> Self {
        let mut this = Self::open(None, false).await.unwrap();

        this.public_worlds = worlds.into_iter().collect();
        this.testing = true;
        this
    }

    pub fn dir(&self) -> &Path {
        self.dir.as_deref().expect(
            "this is Store::test() which is not backed by any directory",
        )
    }

    pub fn create_private_world(
        &self,
        config: WorldConfig,
    ) -> Result<WorldHandle> {
        let permit = self
            .private_worlds_sem
            .clone()
            .try_acquire_owned()
            .map_err(|_| anyhow!("ouch, the server is currently overloaded"))?;

        let handle = kartoffels_world::create(config).with_permit(permit);

        if self.testing {
            self.private_worlds.lock().unwrap().push(handle.clone());
        }

        Ok(handle)
    }

    pub fn first_private_world(&self) -> WorldHandle {
        self.private_worlds.lock().unwrap()[0].clone()
    }

    pub fn public_worlds(&self) -> &[WorldHandle] {
        &self.public_worlds
    }

    pub fn create_session(&self) -> SessionToken {
        let mut rng = rand::thread_rng();
        let mut sessions = self.sessions.lock().unwrap();

        loop {
            let id = SessionId(rng.gen());

            if let hash_map::Entry::Vacant(entry) = sessions.entry(id) {
                entry.insert(Default::default());

                info!(?id, "session created");

                return SessionToken {
                    id,
                    sessions: self.sessions.clone(),
                };
            }
        }
    }

    pub fn with_session<T>(
        &self,
        id: SessionId,
        f: impl FnOnce(&mut Session) -> T,
    ) -> Option<T> {
        self.sessions.lock().unwrap().get_mut(&id).map(f)
    }

    pub fn first_session_id(&self) -> SessionId {
        *self.sessions.lock().unwrap().keys().next().unwrap()
    }

    pub fn testing(&self) -> bool {
        self.testing
    }

    pub async fn close(&self) -> Result<()> {
        for world in &self.public_worlds {
            world.shutdown().await?;
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Session {
    upload: Option<oneshot::Sender<Vec<u8>>>,
}

impl Session {
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

#[derive(Debug)]
pub struct SessionUploadInterest {
    rx: oneshot::Receiver<Vec<u8>>,
}

impl SessionUploadInterest {
    pub fn try_recv(&mut self) -> Option<Vec<u8>> {
        self.rx.try_recv().ok()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(Id);

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct SessionToken {
    id: SessionId,
    sessions: Arc<Mutex<AHashMap<SessionId, Session>>>,
}

impl SessionToken {
    pub fn id(&self) -> SessionId {
        self.id
    }
}

impl Drop for SessionToken {
    fn drop(&mut self) {
        debug!(id = ?self.id, "session dropped");

        self.sessions.lock().unwrap().remove(&self.id);
    }
}
