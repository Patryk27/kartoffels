use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
enum Cmd {
    Serve(kartoffels_server::Cmd),

    Toolbox {
        #[command(subcommand)]
        cmd: kartoffels_toolbox::Cmd,
    },
}

fn main() -> Result<()> {
    match Cmd::parse() {
        Cmd::Serve(cmd) => cmd.run(),
        Cmd::Toolbox { cmd } => cmd.run(),
    }
}
