mod create_world;

pub use self::create_world::*;
use anyhow::{anyhow, Result};
use clap::Parser;
use kartoffels_store::{Session, Store};
use kartoffels_ui::Term;

#[derive(Debug, Parser)]
pub enum Cmd {
    CreateWorld(CreateWorldCmd),
}

impl Cmd {
    pub fn run(
        self,
        store: &Store,
        sess: &Session,
        term: &mut Term,
    ) -> Result<()> {
        if sess.with(|sess| !sess.is_admin()) {
            return Err(anyhow!("insufficient privileges"));
        }

        match self {
            Cmd::CreateWorld(cmd) => cmd.run(store, term),
        }
    }
}
