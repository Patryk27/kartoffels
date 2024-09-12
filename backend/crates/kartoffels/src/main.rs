use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
pub enum Cmd {
    Serve(kartoffels_server::Cmd),
}

fn main() -> Result<()> {
    let cmd = Cmd::parse();

    match cmd {
        Cmd::Serve(cmd) => cmd.run(),
    }
}
