mod set;

pub use self::set::*;
use super::CmdContext;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
pub enum WorldPolicyCmd {
    Set(SetWorldPolicyCmd),
}

impl WorldPolicyCmd {
    pub async fn run(self, ctxt: &mut CmdContext<'_>) -> Result<()> {
        match self {
            Self::Set(cmd) => cmd.run(ctxt).await,
        }
    }
}
