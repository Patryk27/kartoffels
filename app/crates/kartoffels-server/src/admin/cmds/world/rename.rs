use super::CmdContext;
use anyhow::Result;
use clap::Parser;
use kartoffels_store::WorldId;
use kartoffels_utils::Id;
use std::fmt::Write;

#[derive(Debug, Parser)]
pub struct RenameWorldCmd {
    id: Id,
    name: String,
}

impl RenameWorldCmd {
    pub async fn run(self, ctxt: &mut CmdContext<'_>) -> Result<()> {
        ctxt.store
            .rename_world(WorldId::new(self.id), self.name.clone())
            .await?;

        writeln!(ctxt, "ok, world {} renamed into {}", self.id, self.name)?;

        Ok(())
    }
}
