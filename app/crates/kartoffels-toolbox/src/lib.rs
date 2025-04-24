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
            Cmd::Bench(cmd) => cmd.run(),
            Cmd::WorldToJson(cmd) => cmd.run(),
        }
    }
}
