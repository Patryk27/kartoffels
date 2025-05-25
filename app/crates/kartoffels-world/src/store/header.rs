use super::migrations;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::io::Read;

#[derive(Debug, Serialize, Deserialize)]
pub struct Header {
    magic: [u8; 11],
    version: u32,
    padding: u8,
}

impl Header {
    pub fn read(mut buf: impl Read) -> Result<Self> {
        let mut magic = [0; 11];
        let mut version = [0; 4];
        let mut pad = [0; 1];

        buf.read_exact(&mut magic)?;
        buf.read_exact(&mut version)?;
        buf.read_exact(&mut pad)?;

        Ok(Self {
            magic,
            version: u32::from_be_bytes(version),
            padding: pad[0],
        })
    }

    pub fn write(self, buf: &mut Vec<u8>) {
        buf.extend(&self.magic);
        buf.extend(&u32::to_be_bytes(self.version));
        buf.extend(&[self.padding]);
    }

    pub(crate) fn validated(self) -> Result<Self> {
        if self.magic != Self::default().magic {
            return Err(anyhow!("invalid magic value"));
        }

        if self.version > Self::default().version {
            return Err(anyhow!(
                "unsupported version: got {}, expected <= {}",
                self.version,
                migrations::version(),
            ));
        }

        if self.padding != Self::default().padding {
            return Err(anyhow!("invalid padding"));
        }

        Ok(self)
    }

    pub(crate) fn version(&self) -> u32 {
        self.version
    }
}

impl Default for Header {
    fn default() -> Self {
        Self {
            magic: *b"kartoffels:",
            version: migrations::version(),
            padding: 0,
        }
    }
}
