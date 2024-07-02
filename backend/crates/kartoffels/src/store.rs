mod header;
mod migrations;

use self::header::*;
use crate::{Bots, Map, Mode, Policy, Theme, WorldName};
use anyhow::{Context, Result};
use maybe_owned::MaybeOwned;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Cursor, Write};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct SerializedWorld<'a> {
    pub name: MaybeOwned<'a, WorldName>,
    pub mode: MaybeOwned<'a, Mode>,
    pub theme: MaybeOwned<'a, Theme>,
    pub policy: MaybeOwned<'a, Policy>,
    pub map: MaybeOwned<'a, Map>,
    pub bots: MaybeOwned<'a, Bots>,
}

impl SerializedWorld<'_> {
    pub fn load(path: &Path) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut file = BufReader::new(&mut file);

        let header = Header::read(&mut file)
            .context("couldn't read header")?
            .validated()
            .context("couldn't validate header")?;

        let this =
            ciborium::from_reader(&mut file).context("couldn't read state")?;

        let this = migrations::run(header.version(), Header::VERSION, this)
            .context("couldn't migrate state")?;

        let this = ciborium::from_reader({
            let mut buffer = Vec::new();

            ciborium::into_writer(&this, &mut buffer)?;

            Cursor::new(buffer)
        })
        .context("couldn't deserialize state")?;

        Ok(this)
    }

    pub fn store(self, path: &Path) -> Result<()> {
        let path_new = path.with_extension("world.new");

        let result: Result<()> = try {
            let mut file = File::create(&path_new)?;
            let mut file = BufWriter::new(&mut file);

            Header::default()
                .write(&mut file)
                .context("couldn't write header")?;

            ciborium::into_writer(&self, &mut file)
                .context("couldn't write state")?;

            file.flush()?;
        };

        result.with_context(|| {
            format!("couldn't write: {}", path_new.display())
        })?;

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
