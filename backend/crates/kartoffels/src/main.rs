use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
pub enum Cmd {
    Serve(kartoffels_server::Cmd),

    Toolbox {
        #[clap(subcommand)]
        cmd: kartoffels_toolbox::Cmd,
    },
}

fn main() -> Result<()> {
    let cmd = Cmd::parse();

    match cmd {
        Cmd::Serve(cmd) => cmd.run(),
        Cmd::Toolbox { cmd } => cmd.run(),
    }
}
