use crate::{Bots, Map, Mode, Theme, WorldName};
use anyhow::{Context, Result};
use maybe_owned::MaybeOwned;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct SerializedWorld<'a> {
    // TODO version
    pub name: MaybeOwned<'a, WorldName>,
    pub mode: MaybeOwned<'a, Mode>,
    pub theme: MaybeOwned<'a, Theme>,
    pub map: MaybeOwned<'a, Map>,
    pub bots: MaybeOwned<'a, Bots>,
}

impl SerializedWorld<'_> {
    pub fn load(path: &Path) -> Result<Self> {
        let mut file = File::open(path)
            .with_context(|| format!("couldn't open {}", path.display()))?;

        let mut file = BufReader::new(&mut file);
        let this = ciborium::from_reader(&mut file)?;

        Ok(this)
    }

    pub fn store(self, path: &Path) -> Result<()> {
        let path_new = path.with_extension("world.new");

        let mut file = File::create(&path_new).with_context(|| {
            format!("couldn't create {}", path_new.display())
        })?;

        let mut file = BufWriter::new(&mut file);

        ciborium::into_writer(&self, &mut file)
            .context("couldn't serialize")?;

        file.flush().with_context(|| {
            format!("couldn't flush {}", path_new.display())
        })?;

        drop(file);

        fs::rename(&path_new, path).with_context(|| {
            format!(
                "couldn't rename {} to {}",
                path_new.display(),
                path.display()
            )
        })?;

        Ok(())
    }
}
