use anyhow::Result;
use clap::Parser;
use kartoffels_store::Store;
use kartoffels_utils::Id;

#[derive(Debug, Parser)]
pub struct RenameWorldCmd {
    id: Id,
    name: String,
}

impl RenameWorldCmd {
    pub(super) async fn run(self, store: &Store) -> Result<()> {
        store.rename_world(self.id, self.name).await?;

        Ok(())
    }
}
