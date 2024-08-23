use crate::DecodedWorld;
use anyhow::{Context, Result};
use clap::Parser;
use kartoffels::prelude::json_to_cbor;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct EncodeWorldCmd {
    src: PathBuf,

    #[clap(long)]
    dst: Option<PathBuf>,
}

impl EncodeWorldCmd {
    pub fn run(self) -> Result<()> {
        let src = {
            let src = File::open(&self.src).with_context(|| {
                format!("couldn't open {}", self.src.display())
            })?;

            BufReader::new(src)
        };

        let src: DecodedWorld = serde_json::from_reader(src)?;

        let mut dst = {
            let dst_path =
                self.dst.unwrap_or_else(|| self.src.with_extension("world"));

            let dst = File::create(&dst_path).with_context(|| {
                format!("couldn't create {}", dst_path.display())
            })?;

            BufWriter::new(dst)
        };

        let payload = json_to_cbor(src.payload);

        src.header.write(&mut dst)?;
        ciborium::into_writer(&payload, &mut dst)?;

        Ok(())
    }
}
