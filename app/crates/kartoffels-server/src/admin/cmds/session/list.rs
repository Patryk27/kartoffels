use super::CmdContext;
use anyhow::Result;
use clap::Parser;
use prettytable::{Table, row};
use std::fmt::Write;

#[derive(Debug, Parser)]
pub struct ListSessionsCmd;

impl ListSessionsCmd {
    pub async fn run(self, ctxt: &mut CmdContext<'_>) -> Result<()> {
        let mut table = Table::new();

        table.add_row(row!["id", "role", "created-at"]);

        for sess in ctxt.store.find_sessions(None).await? {
            let id = sess.id();

            sess.with(|sess| {
                table.add_row(row![id, sess.role(), sess.created_at()]);
            });
        }

        writeln!(ctxt, "{table}")?;

        Ok(())
    }
}
