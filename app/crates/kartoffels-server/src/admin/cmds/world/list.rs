use super::CmdContext;
use anyhow::Result;
use clap::Parser;
use kartoffels_store::WorldVis;
use prettytable::{Table, row};
use std::fmt::Write;

#[derive(Debug, Parser)]
pub struct ListWorldsCmd {
    #[clap(short, long = "vis")]
    vis: Option<WorldVis>,
}

impl ListWorldsCmd {
    pub async fn run(self, ctxt: &mut CmdContext<'_>) -> Result<()> {
        let mut table = Table::new();

        table.add_row(row!["id", "vis", "name"]);

        for world in ctxt.store.find_worlds(self.vis).await? {
            table.add_row(row![world.id(), world.vis(), world.name()]);
        }

        writeln!(ctxt, "{table}")?;

        Ok(())
    }
}
