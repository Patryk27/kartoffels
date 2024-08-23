use crate::DecodedWorld;
use anyhow::{Context, Result};
use clap::Parser;
use kartoffels::prelude::{cbor_to_json, Header};
use std::fs::{self, File};
use std::io::BufReader;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct DecodeWorldCmd {
    src: PathBuf,

    #[clap(long)]
    dst: Option<PathBuf>,
}

impl DecodeWorldCmd {
    pub fn run(self) -> Result<()> {
        let mut src = {
            let src = File::open(&self.src).with_context(|| {
                format!("couldn't open {}", self.src.display())
            })?;

            BufReader::new(src)
        };

        let header = Header::read(&mut src)?;
        let payload = ciborium::from_reader(src)?;
        let payload = cbor_to_json(payload);

        let dst = {
            let dst = DecodedWorld { header, payload };

            serde_json::to_string_pretty(&dst)
                .context("couldn't serialize to json")?
        };

        let dst_path =
            self.dst.unwrap_or_else(|| self.src.with_extension("json"));

        fs::write(&dst_path, dst).with_context(|| {
            format!("couldn't write to {}", dst_path.display())
        })?;

        Ok(())
    }
}
