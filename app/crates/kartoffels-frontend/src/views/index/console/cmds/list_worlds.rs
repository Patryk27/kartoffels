use anyhow::Result;
use clap::Parser;
use kartoffels_store::{Store, WorldType};
use kartoffels_ui::Term;
use std::fmt::Write;

#[derive(Debug, Parser)]
pub struct ListWorldsCmd {
    #[clap(short, long = "type")]
    ty: Option<WorldType>,
}

impl ListWorldsCmd {
    pub(super) fn run(self, store: &Store, term: &mut Term) -> Result<()> {
        for (ty, handle) in store.worlds(self.ty) {
            writeln!(term, "{}: {} ({ty})", handle.id(), handle.name())?;
        }

        Ok(())
    }
}
