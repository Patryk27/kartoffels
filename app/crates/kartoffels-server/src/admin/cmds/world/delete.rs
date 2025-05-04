use super::CmdContext;
use anyhow::Result;
use clap::Parser;
use kartoffels_store::WorldId;
use kartoffels_utils::Id;
use std::fmt::Write;

#[derive(Debug, Parser)]
pub struct DeleteWorldCmd {
    id: Id,
}

impl DeleteWorldCmd {
    pub async fn run(self, ctxt: &mut CmdContext<'_>) -> Result<()> {
        ctxt.store.delete_world(WorldId::new(self.id)).await?;

        writeln!(ctxt, "ok, world {} deleted", self.id)?;

        Ok(())
    }
}
