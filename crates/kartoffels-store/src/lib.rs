#![feature(map_try_insert)]
#![feature(try_blocks)]

use anyhow::{Context, Result};
use kartoffels_utils::Id;
use kartoffels_world::prelude::Handle as WorldHandle;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::info;

#[derive(Debug)]
pub struct Store {
    pub dir: PathBuf,
    pub worlds: Vec<(Id, WorldHandle)>,
}

impl Store {
    pub async fn open(dir: &Path) -> Result<Self> {
        info!(?dir, "opening");

        let mut worlds = Vec::new();
        let mut entries = fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();

            let Some(entry_stem) =
                entry_path.file_stem().and_then(|stem| stem.to_str())
            else {
                continue;
            };

            let Some("world") =
                entry_path.extension().and_then(|ext| ext.to_str())
            else {
                continue;
            };

            info!("loading: {}", entry_path.display());

            let result: Result<()> = try {
                let id = entry_stem
                    .parse()
                    .context("couldn't extract world id from path")?;

                let handle = kartoffels_world::spawn(&entry_path)?;

                worlds.push((id, handle));
            };

            result.with_context(|| {
                format!("couldn't resume world: {}", entry_path.display())
            })?;
        }

        worlds.sort_by_key(|(_, world)| world.name().to_owned());

        info!("ready");

        Ok(Self {
            dir: dir.to_owned(),
            worlds,
        })
    }
}
