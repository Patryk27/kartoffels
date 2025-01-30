mod cmds;

use self::cmds::*;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
pub enum Cmd {
    WorldToJson(WorldToJsonCmd),
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        match self {
            Cmd::WorldToJson(cmd) => cmd.run(),
        }
    }
}
