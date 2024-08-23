mod cmds;

use self::cmds::*;
use anyhow::Result;
use clap::Subcommand;
use kartoffels::prelude::Header;
use serde::{Deserialize, Serialize};

#[derive(Debug, Subcommand)]
pub enum Cmd {
    DecodeWorld(DecodeWorldCmd),
    EncodeWorld(EncodeWorldCmd),
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        match self {
            Cmd::DecodeWorld(cmd) => cmd.run(),
            Cmd::EncodeWorld(cmd) => cmd.run(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DecodedWorld {
    header: Header,
    payload: serde_json::Value,
}
