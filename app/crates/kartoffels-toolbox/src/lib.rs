mod cmds;

use self::cmds::*;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
pub enum Cmd {
    Bench(BenchCmd),
    WorldToJson(WorldToJsonCmd),
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        match self {
            Self::Bench(cmd) => cmd.run(),
            Self::WorldToJson(cmd) => cmd.run(),
        }
    }
}
