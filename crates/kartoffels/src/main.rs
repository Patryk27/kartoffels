#![feature(map_try_insert)]

mod cmds;

use self::cmds::*;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
pub enum Cmd {
    Serve(ServeCmd),
}

fn main() -> Result<()> {
    let cmd = Cmd::parse();

    match cmd {
        Cmd::Serve(cmd) => cmd.run(),
    }
}
