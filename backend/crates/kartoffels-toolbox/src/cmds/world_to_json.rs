use anyhow::{Context, Result};
use clap::Parser;
use kartoffels_utils::cbor_to_json;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct WorldToJsonCmd {
    src: PathBuf,

    #[clap(long)]
    dst: Option<PathBuf>,
}

impl WorldToJsonCmd {
    pub(crate) fn run(self) -> Result<()> {
        let dst_path =
            self.dst.unwrap_or_else(|| self.src.with_extension("json"));

        let src = ciborium::from_reader({
            let src = File::open(&self.src).with_context(|| {
                format!("couldn't read from {}", self.src.display())
            })?;

            let mut src = BufReader::new(src);
            let mut header = [0; 16];

            src.read_exact(&mut header)?;
            src
        })?;

        let dst = cbor_to_json(src, true);

        let dst = serde_json::to_string_pretty(&dst)
            .context("couldn't serialize to json")?;

        fs::write(&dst_path, dst).with_context(|| {
            format!("couldn't write to {}", dst_path.display())
        })?;

        Ok(())
    }
}
