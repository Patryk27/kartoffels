#![feature(let_chains)]
#![feature(try_blocks)]

mod secret;
mod session;
mod sessions;
mod world;
mod worlds;

pub use self::secret::*;
pub use self::session::*;
use self::sessions::*;
pub use self::world::*;
use self::worlds::*;
use anyhow::Result;
use kartoffels_utils::Id;
use kartoffels_world::prelude::{
    Clock, Config as WorldConfig, Handle as WorldHandle,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;

#[derive(Debug)]
pub struct Store {
    dir: Option<PathBuf>,
    secret: Option<Secret>,
    worlds: Worlds,
    sessions: Sessions,
    testing: bool,
}

impl Store {
    pub async fn new(
        dir: Option<&Path>,
        secret: Option<Secret>,
    ) -> Result<Self> {
        info!("opening");

        Ok(Self {
            secret,
            worlds: Worlds::new(dir).await?,
            dir: dir.map(|dir| dir.to_owned()),
            sessions: Default::default(),
            testing: false,
        })
    }

    pub async fn test(worlds: impl IntoIterator<Item = WorldHandle>) -> Self {
        let secret = "foobar".parse().unwrap();
        let mut this = Self::new(None, Some(secret)).await.unwrap();

        this.worlds.set(worlds);
        this.testing = true;
        this
    }

    pub fn dir(&self) -> &Path {
        self.dir.as_deref().unwrap()
    }

    pub fn secret(&self) -> Option<&str> {
        self.secret.as_ref().map(|secret| secret.as_str())
    }

    // ---

    pub fn create_public_world(
        &self,
        config: WorldConfig,
    ) -> Result<WorldHandle> {
        self.worlds.create(
            self.testing,
            self.dir.as_deref(),
            WorldType::Public,
            config,
        )
    }

    pub fn create_private_world(
        &self,
        config: WorldConfig,
    ) -> Result<WorldHandle> {
        self.worlds.create(
            self.testing,
            self.dir.as_deref(),
            WorldType::Private,
            config,
        )
    }

    pub async fn rename_world(&self, id: Id, name: String) -> Result<()> {
        self.worlds.rename(id, name).await
    }

    pub async fn delete_world(&self, id: Id) -> Result<()> {
        self.worlds.delete(self.dir.as_deref(), id).await
    }

    pub fn worlds(
        &self,
        ty: Option<WorldType>,
    ) -> Vec<(WorldType, WorldHandle)> {
        self.worlds.list(ty)
    }

    pub fn public_worlds(&self) -> Arc<Vec<WorldHandle>> {
        self.worlds.public()
    }

    pub fn first_private_world(&self) -> WorldHandle {
        assert!(self.testing);

        self.worlds.first_private().unwrap()
    }

    pub fn world_config(&self, name: &str) -> WorldConfig {
        if self.testing() {
            WorldConfig {
                clock: Clock::manual(),
                events: true,
                name: name.into(),
                seed: Some(Default::default()),
                ..Default::default()
            }
        } else {
            WorldConfig {
                events: true,
                name: name.into(),
                ..Default::default()
            }
        }
    }

    // ---

    pub fn create_session(&self) -> Session {
        self.sessions.create(&mut rand::thread_rng())
    }

    pub fn first_session_id(&self) -> SessionId {
        self.sessions.first_id().unwrap()
    }

    pub fn with_session<T>(
        &self,
        id: SessionId,
        f: impl FnOnce(&mut SessionEntry) -> T,
    ) -> Option<T> {
        self.sessions.with(id, f)
    }

    pub fn testing(&self) -> bool {
        self.testing
    }

    pub async fn close(&self) -> Result<()> {
        self.worlds.shutdown().await?;

        Ok(())
    }
}
