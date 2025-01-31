use anyhow::Result;
use clap::Parser;
use kartoffels_store::Store;
use kartoffels_utils::Id;

#[derive(Debug, Parser)]
pub struct DeleteWorldCmd {
    id: Id,
}

impl DeleteWorldCmd {
    pub(super) async fn run(self, store: &Store) -> Result<()> {
        store.delete_world(self.id).await?;

        Ok(())
    }
}
