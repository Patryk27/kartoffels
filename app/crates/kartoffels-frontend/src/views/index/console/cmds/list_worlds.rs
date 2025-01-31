use anyhow::Result;
use clap::Parser;
use kartoffels_store::Store;
use kartoffels_ui::Term;
use std::fmt::Write;

#[derive(Debug, Parser)]
pub struct ListWorldsCmd;

impl ListWorldsCmd {
    pub(super) fn run(self, store: &Store, term: &mut Term) -> Result<()> {
        for (ty, handle) in store.worlds() {
            writeln!(term, "{}: {} ({ty:?})", handle.id(), handle.name())?;
        }

        Ok(())
    }
}
