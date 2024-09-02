#![feature(map_try_insert)]
#![feature(try_blocks)]

use anyhow::{Context, Result};
use kartoffels_world::prelude::Handle as WorldHandle;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::info;

#[derive(Debug)]
pub struct Store {
    pub dir: PathBuf,
    pub worlds: Vec<WorldHandle>,
}

impl Store {
    pub async fn open(dir: &Path) -> Result<Self> {
        info!(?dir, "opening");

        let mut worlds = Vec::new();
        let mut entries = fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();

            let Some("world") =
                entry_path.extension().and_then(|ext| ext.to_str())
            else {
                continue;
            };

            info!("loading: {}", entry_path.display());

            let world =
                kartoffels_world::spawn(&entry_path).with_context(|| {
                    format!("couldn't spawn world: {}", entry_path.display())
                })?;

            worlds.push(world);
        }

        worlds.sort_by_key(|world| world.name().to_owned());

        info!("ready");

        Ok(Self {
            dir: dir.to_owned(),
            worlds,
        })
    }

    pub async fn close(&self) -> Result<()> {
        for world in &self.worlds {
            world.shutdown().await?;
        }

        Ok(())
    }
}
