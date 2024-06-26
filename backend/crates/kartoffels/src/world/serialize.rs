use crate::{Bots, Map, Mode, Policy, Theme, WorldName};
use anyhow::{anyhow, Context, Result};
use maybe_owned::MaybeOwned;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
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

        Header::read(&mut file)
            .context("couldn't read header")?
            .validate()
            .context("couldn't validate header")?;

        let this = ciborium::from_reader(&mut file)?;

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

#[derive(Debug)]
struct Header {
    magic: [u8; 11],
    version: u32,
    padding: u8,
}

impl Header {
    const VERSION: u32 = 1;

    fn read(mut reader: impl Read) -> Result<Self> {
        let mut magic = [0; 11];
        let mut version = [0; 4];
        let mut pad = [0; 1];

        reader.read_exact(&mut magic)?;
        reader.read_exact(&mut version)?;
        reader.read_exact(&mut pad)?;

        Ok(Self {
            magic,
            version: u32::from_be_bytes(version),
            padding: pad[0],
        })
    }

    fn write(self, mut writer: impl Write) -> Result<()> {
        writer.write_all(&self.magic)?;
        writer.write_all(&u32::to_be_bytes(self.version))?;
        writer.write_all(&[self.padding])?;

        Ok(())
    }

    fn validate(self) -> Result<()> {
        if self.magic != Self::default().magic {
            return Err(anyhow!("invalid magic value"));
        }

        if self.version != Self::default().version {
            return Err(anyhow!(
                "version mismatch: expected {}, got {}",
                Self::VERSION,
                self.version
            ));
        }

        if self.padding != Self::default().padding {
            return Err(anyhow!("invalid padding"));
        }

        Ok(())
    }
}

impl Default for Header {
    fn default() -> Self {
        Self {
            magic: *b"kartoffels:",
            version: Self::VERSION,
            padding: 0,
        }
    }
}
