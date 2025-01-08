use crate::storage::{migrations, Header};
use crate::SerializedWorld;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;

pub fn load(path: &Path) -> Result<SerializedWorld<'static>> {
    let mut file = File::open(path)?;
    let mut file = BufReader::new(&mut file);

    let header = Header::read(&mut file)
        .context("couldn't read header")?
        .validated()
        .context("couldn't validate header")?;

    let this =
        ciborium::from_reader(&mut file).context("couldn't read state")?;

    let this = migrations::run(header.version(), migrations::version(), this)
        .context("couldn't migrate state")?;

    let this = ciborium::from_reader({
        let mut buffer = Vec::new();

        ciborium::into_writer(&this, &mut buffer)?;

        Cursor::new(buffer)
    })
    .context("couldn't deserialize state")?;

    Ok(this)
}
