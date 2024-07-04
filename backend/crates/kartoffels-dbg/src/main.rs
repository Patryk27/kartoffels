use anyhow::{Context, Result};
use clap::Parser;
use kartoffels::iface::cbor_to_json;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::PathBuf;

#[derive(Debug, Parser)]
enum Cmd {
    WorldToJson {
        src: PathBuf,

        #[clap(long)]
        dst: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    match Cmd::parse() {
        Cmd::WorldToJson {
            src: src_path,
            dst: dst_path,
        } => {
            let dst_path =
                dst_path.unwrap_or_else(|| src_path.with_extension("json"));

            let src = ciborium::from_reader({
                let src = File::open(&src_path).with_context(|| {
                    format!("couldn't read from {}", src_path.display())
                })?;

                let mut src = BufReader::new(src);
                let mut header = [0; 16];

                src.read_exact(&mut header)?;
                src
            })?;

            let dst = cbor_to_json(src);

            let dst = serde_json::to_string_pretty(&dst)
                .context("couldn't serialize to json")?;

            fs::write(&dst_path, dst).with_context(|| {
                format!("couldn't write to {}", dst_path.display())
            })?;

            Ok(())
        }
    }
}
