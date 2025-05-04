use super::CmdContext;
use anyhow::Result;
use clap::Parser;
use kartoffels_store::{SessionId, SessionRole};
use kartoffels_utils::Id;
use std::fmt::Write;

#[derive(Debug, Parser)]
pub struct PromoteSessionCmd {
    id: Id,
}

impl PromoteSessionCmd {
    pub async fn run(self, ctxt: &mut CmdContext<'_>) -> Result<()> {
        ctxt.store
            .get_session(SessionId::new(self.id))
            .await?
            .with(|sess| {
                *sess.role_mut() = SessionRole::Admin;
            });

        writeln!(ctxt, "ok, session {} promoted", self.id)?;

        Ok(())
    }
}
