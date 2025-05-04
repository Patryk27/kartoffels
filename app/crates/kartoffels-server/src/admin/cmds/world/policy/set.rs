use super::CmdContext;
use anyhow::{Context, Result};
use clap::Parser;
use kartoffels_store::WorldId;
use kartoffels_utils::Id;
use std::fmt::Write;

#[derive(Debug, Parser)]
pub struct SetWorldPolicyCmd {
    id: Id,
    policy: String,
}

impl SetWorldPolicyCmd {
    pub async fn run(self, ctxt: &mut CmdContext<'_>) -> Result<()> {
        let policy = serde_json::from_str(&self.policy).with_context(|| {
            format!("couldn't parse policy: {}", self.policy)
        })?;

        ctxt.store
            .get_world(WorldId::new(self.id))
            .await?
            .set_policy(policy)
            .await?;

        writeln!(ctxt, "ok, policy changed")?;

        Ok(())
    }
}
