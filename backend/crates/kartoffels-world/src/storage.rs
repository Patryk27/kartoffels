mod header;
mod migrations;
mod systems;

use self::header::*;
pub use self::systems::*;
use crate::{Bots, Clock, Map, Metronome, Mode, Policy, Theme};
use anyhow::{Context, Result};
use maybe_owned::MaybeOwned;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::future::Future;
use std::io::{BufReader, Cursor};
use std::path::Path;
use std::time::Duration;
use tokio::task;

const VERSION: u32 = 8;

#[derive(Serialize, Deserialize)]
pub struct SerializedWorld<'a> {
    pub bots: MaybeOwned<'a, Bots>,
    pub clock: MaybeOwned<'a, Clock>,
    pub map: MaybeOwned<'a, Map>,
    pub mode: MaybeOwned<'a, Mode>,
    pub name: MaybeOwned<'a, String>,
    pub policy: MaybeOwned<'a, Policy>,
    pub theme: MaybeOwned<'a, Theme>,
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

        let this = migrations::run(header.version(), VERSION, this)
            .context("couldn't migrate state")?;

        let this = ciborium::from_reader({
            let mut buffer = Vec::new();

            ciborium::into_writer(&this, &mut buffer)?;

            Cursor::new(buffer)
        })
        .context("couldn't deserialize state")?;

        Ok(this)
    }

    pub fn store(
        self,
        path: &Path,
    ) -> Result<impl Future<Output = Result<(Duration, Duration)>>> {
        // Serializing directly into the file would be faster, but it also makes
        // the event loop potentially I/O bound, so let's first serialize into a
        // buffer and then move the I/O onto a thread pool.

        let (buffer, tt_ser) = Metronome::try_measure(|| {
            let mut buffer = Vec::new();

            Header::default()
                .write(&mut buffer)
                .context("couldn't write header")?;

            ciborium::into_writer(&self, &mut buffer)
                .context("couldn't write state")?;

            Ok(buffer)
        })?;

        let path = path.to_owned();
        let path_new = path.with_extension("world.new");

        let task = task::spawn_blocking(move || {
            let (_, tt_io) = Metronome::try_measure(|| {
                fs::write(&path_new, &buffer).with_context(|| {
                    format!("couldn't write: {}", path_new.display())
                })?;

                fs::rename(&path_new, &path).with_context(|| {
                    format!(
                        "couldn't rename {} to {}",
                        path_new.display(),
                        path.display()
                    )
                })?;

                Ok(())
            })?;

            Ok((tt_ser, tt_io))
        });

        Ok(async move { task.await.context("task crashed")? })
    }
}
