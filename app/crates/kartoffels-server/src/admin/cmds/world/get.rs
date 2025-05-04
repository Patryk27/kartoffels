use super::CmdContext;
use anyhow::Result;
use clap::Parser;
use kartoffels_store::WorldId;
use kartoffels_utils::Id;
use std::fmt::Write;

#[derive(Debug, Parser)]
pub struct GetWorldCmd {
    id: Id,
}

impl GetWorldCmd {
    pub async fn run(self, ctxt: &mut CmdContext<'_>) -> Result<()> {
        let world = ctxt.store.get_world(WorldId::new(self.id)).await?;

        writeln!(ctxt, "id:")?;
        writeln!(ctxt, "{}", world.id())?;
        writeln!(ctxt)?;

        writeln!(ctxt, "name:")?;
        writeln!(ctxt, "{}", world.name())?;
        writeln!(ctxt)?;

        writeln!(ctxt, "policy:")?;
        writeln!(
            ctxt,
            "{}",
            serde_json::to_string(&world.get_policy().await?)?,
        )?;

        Ok(())
    }
}
