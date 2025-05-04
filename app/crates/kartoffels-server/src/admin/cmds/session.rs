mod list;
mod promote;

pub use self::list::*;
pub use self::promote::*;
use super::CmdContext;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
pub enum SessionCmd {
    List(ListSessionsCmd),
    Promote(PromoteSessionCmd),
}

impl SessionCmd {
    pub async fn run(self, ctxt: &mut CmdContext<'_>) -> Result<()> {
        match self {
            Self::List(cmd) => cmd.run(ctxt).await,
            Self::Promote(cmd) => cmd.run(ctxt).await,
        }
    }
}
