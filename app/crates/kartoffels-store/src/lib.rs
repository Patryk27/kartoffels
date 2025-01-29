#![feature(try_blocks)]

mod open;
mod session;

pub use self::session::*;
use ahash::AHashMap;
use anyhow::{anyhow, Result};
use kartoffels_world::prelude::{
    Clock, Config as WorldConfig, Handle as WorldHandle,
};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;
use tracing::info;

#[derive(Debug)]
pub struct Store {
    dir: Option<PathBuf>,
    secret: Option<String>,
    public_worlds: Vec<WorldHandle>,
    private_worlds: Mutex<Vec<WorldHandle>>,
    private_worlds_sem: Arc<Semaphore>,
    sessions: Arc<Mutex<AHashMap<SessionId, Arc<Mutex<SessionEntry>>>>>,
    testing: bool,
}

impl Store {
    pub const MAX_SECRET_LENGTH: usize = 64;

    pub async fn open(
        dir: Option<&Path>,
        secret: Option<String>,
    ) -> Result<Self> {
        info!("opening");

        let public_worlds = open::load_worlds(dir).await?;

        if let Some(secret) = &secret {
            if secret.len() > Self::MAX_SECRET_LENGTH {
                return Err(anyhow!(
                    "secret is too long - the limit is {} characters",
                    Self::MAX_SECRET_LENGTH
                ));
            }

            if secret.chars().any(|ch| ch.is_ascii_control()) {
                return Err(anyhow!("secret contains forbidden characters"));
            }
        }

        info!("ready");

        Ok(Self {
            dir: dir.map(|dir| dir.to_owned()),
            secret,
            public_worlds,
            private_worlds: Default::default(),
            private_worlds_sem: Arc::new(Semaphore::new(128)), // TODO make configurable
            sessions: Default::default(),
            testing: false,
        })
    }

    pub async fn test(worlds: impl IntoIterator<Item = WorldHandle>) -> Self {
        let mut this = Self::open(None, None).await.unwrap();

        this.public_worlds = worlds.into_iter().collect();
        this.testing = true;
        this
    }

    pub fn dir(&self) -> &Path {
        self.dir.as_deref().unwrap()
    }

    pub fn secret(&self) -> Option<&str> {
        self.secret.as_deref()
    }

    pub fn with_session<T>(
        &self,
        id: SessionId,
        f: impl FnOnce(&mut SessionEntry) -> T,
    ) -> Option<T> {
        self.sessions
            .lock()
            .unwrap()
            .get(&id)
            .map(|sess| f(&mut sess.lock().unwrap()))
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

    pub fn create_session(&self) -> Session {
        Session::create(self.sessions.clone())
    }

    pub fn first_session_id(&self) -> SessionId {
        *self.sessions.lock().unwrap().keys().next().unwrap()
    }

    pub fn testing(&self) -> bool {
        self.testing
    }

    pub fn world_config(&self, name: &str) -> WorldConfig {
        if self.testing() {
            WorldConfig {
                clock: Clock::manual(),
                emit_events: true,
                name: name.into(),
                seed: Some(Default::default()),
                ..Default::default()
            }
        } else {
            WorldConfig {
                emit_events: true,
                name: name.into(),
                ..Default::default()
            }
        }
    }

    pub async fn close(&self) -> Result<()> {
        for world in &self.public_worlds {
            world.shutdown().await?;
        }

        Ok(())
    }
}
