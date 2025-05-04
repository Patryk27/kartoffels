mod create;
mod delete;
mod get;
mod list;
mod policy;
mod rename;

pub use self::create::*;
pub use self::delete::*;
pub use self::get::*;
pub use self::list::*;
pub use self::policy::*;
pub use self::rename::*;
use super::CmdContext;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
pub enum WorldCmd {
    Create(CreateWorldCmd),
    Delete(DeleteWorldCmd),
    Get(GetWorldCmd),
    List(ListWorldsCmd),
    Rename(RenameWorldCmd),

    #[clap(subcommand)]
    Policy(WorldPolicyCmd),
}

impl WorldCmd {
    pub async fn run(self, ctxt: &mut CmdContext<'_>) -> Result<()> {
        match self {
            Self::Create(cmd) => cmd.run(ctxt).await,
            Self::Delete(cmd) => cmd.run(ctxt).await,
            Self::Get(cmd) => cmd.run(ctxt).await,
            Self::List(cmd) => cmd.run(ctxt).await,
            Self::Rename(cmd) => cmd.run(ctxt).await,

            Self::Policy(cmd) => cmd.run(ctxt).await,
        }
    }
}
