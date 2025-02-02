mod create_world;
mod delete_world;
mod list_worlds;

pub use self::create_world::*;
pub use self::delete_world::*;
pub use self::list_worlds::*;
use anyhow::{anyhow, Result};
use clap::Parser;
use kartoffels_store::{Session, Store};
use kartoffels_ui::Term;
use std::ops::ControlFlow;

#[derive(Debug, Parser)]
pub enum Cmd {
    CreateWorld(CreateWorldCmd),
    DeleteWorld(DeleteWorldCmd),
    ListWorlds(ListWorldsCmd),

    Exit,
}

impl Cmd {
    pub async fn run(
        self,
        store: &Store,
        sess: &Session,
        term: &mut Term,
    ) -> Result<ControlFlow<()>> {
        if sess.with(|sess| !sess.is_admin()) {
            return Err(anyhow!("insufficient privileges"));
        }

        match self {
            Cmd::CreateWorld(cmd) => cmd.run(store, term)?,
            Cmd::DeleteWorld(cmd) => cmd.run(store).await?,
            Cmd::ListWorlds(cmd) => cmd.run(store, term)?,

            Cmd::Exit => {
                return Ok(ControlFlow::Break(()));
            }
        }

        Ok(ControlFlow::Continue(()))
    }
}
