#![feature(try_blocks)]

use anyhow::{Context, Result};
use kartoffels_world::prelude::{Config as WorldConfig, Handle as WorldHandle};
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
            let path = entry.path();

            let Some(entry_stem) =
                path.file_stem().and_then(|stem| stem.to_str())
            else {
                continue;
            };

            let Some("world") = path.extension().and_then(|ext| ext.to_str())
            else {
                continue;
            };

            info!("loading: {}", path.display());

            let result: Result<()> = try {
                let id = entry_stem
                    .parse()
                    .context("couldn't extract world id from path")?;

                let world = kartoffels_world::resume(id, &path)?;

                worlds.push(world);
            };

            result.with_context(|| {
                format!("couldn't resume world: {}", path.display())
            })?;
        }

        worlds.sort_by_key(|world| world.name().to_owned());

        info!("ready");

        Ok(Self {
            dir: dir.to_owned(),
            worlds,
        })
    }

    pub fn create_world(&self, config: WorldConfig) -> WorldHandle {
        // TODO add limits
        kartoffels_world::create(config)
    }

    pub async fn close(&self) -> Result<()> {
        for world in &self.worlds {
            world.shutdown().await?;
        }

        Ok(())
    }
}
