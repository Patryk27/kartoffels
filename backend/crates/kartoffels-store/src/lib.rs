#![feature(try_blocks)]

mod open;

use anyhow::{anyhow, Result};
use kartoffels_world::prelude::{Config as WorldConfig, Handle as WorldHandle};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;
use tracing::info;

#[derive(Debug)]
pub struct Store {
    pub dir: Option<PathBuf>,
    pub worlds: StoreWorlds,
    pub testing: bool,
}

impl Store {
    pub async fn open(dir: Option<&Path>, bench: bool) -> Result<Self> {
        info!("opening");

        let worlds = StoreWorlds {
            public: open::load_worlds(dir, bench).await?,
            private: Default::default(),
            semaphore: Arc::new(Semaphore::new(128)), // TODO make configurable
        };

        info!("ready");

        Ok(Self {
            dir: dir.map(|dir| dir.to_owned()),
            worlds,
            testing: false,
        })
    }

    pub async fn test() -> Self {
        let mut this = Self::open(None, false).await.unwrap();

        this.testing = true;
        this
    }

    pub fn create_world(&self, config: WorldConfig) -> Result<WorldHandle> {
        let permit =
            self.worlds.semaphore.clone().try_acquire_owned().map_err(
                |_| anyhow!("ouch, the server is currently overloaded"),
            )?;

        let handle = kartoffels_world::create(config).with_permit(permit);

        if self.testing {
            self.worlds.private.lock().unwrap().push(handle.clone());
        }

        Ok(handle)
    }

    pub async fn close(&self) -> Result<()> {
        for world in &self.worlds.public {
            world.shutdown().await?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct StoreWorlds {
    pub public: Vec<WorldHandle>,
    pub private: Mutex<Vec<WorldHandle>>,
    pub semaphore: Arc<Semaphore>,
}

impl StoreWorlds {
    pub fn first_private(&self) -> WorldHandle {
        self.private.lock().unwrap()[0].clone()
    }
}
