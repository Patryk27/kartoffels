use crate::{Bots, Map, Mode, Theme, WorldName};
use anyhow::Result;
use maybe_owned::MaybeOwned;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};

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
    pub fn load(file: &mut File) -> Result<Self> {
        let mut reader = BufReader::new(&mut *file);
        let this = ciborium::from_reader(&mut reader)?;

        Ok(this)
    }

    pub fn store(self, file: &mut File) -> Result<()> {
        file.seek(SeekFrom::Start(0))?;

        let mut writer = BufWriter::new(&mut *file);

        ciborium::into_writer(&self, &mut writer)?;

        writer.flush()?;
        drop(writer);

        let len = file.stream_position()?;

        file.set_len(len)?;

        Ok(())
    }
}
