use anyhow::{Context, Result};
use kartoffels_world::prelude::Handle;
use std::path::Path;
use tokio::fs;
use tracing::info;

pub async fn load_worlds(
    dir: Option<&Path>,
    bench: bool,
) -> Result<Vec<Handle>> {
    let Some(dir) = dir else {
        return Ok(Default::default());
    };

    let mut worlds = Vec::new();
    let mut entries = fs::read_dir(dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        let Some(entry_stem) = path.file_stem().and_then(|stem| stem.to_str())
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

            let world = kartoffels_world::resume(id, &path, bench)?;

            worlds.push(world);
        };

        result.with_context(|| {
            format!("couldn't resume world: {}", path.display())
        })?;
    }

    worlds.sort_by_key(|world| world.name().to_owned());

    Ok(worlds)
}
