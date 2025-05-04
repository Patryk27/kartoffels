mod session;
mod world;

use self::session::*;
use self::world::*;
use anyhow::Result;
use clap::Parser;
use kartoffels_store::Store;
use russh::server::Session;
use russh::{ChannelId, CryptoVec};
use std::fmt;

#[derive(Debug, Parser)]
pub enum Cmd {
    #[command(subcommand)]
    Session(SessionCmd),
    #[command(subcommand)]
    World(WorldCmd),
}

impl Cmd {
    pub async fn run(self, ctxt: &mut CmdContext<'_>) -> Result<()> {
        match self {
            Self::Session(cmd) => cmd.run(ctxt).await,
            Self::World(cmd) => cmd.run(ctxt).await,
        }
    }
}

#[derive(Debug)]
pub struct CmdContext<'a> {
    pub store: &'a Store,
    pub chan: ChannelId,
    pub sess: &'a mut Session,
}

impl fmt::Write for CmdContext<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.sess
            .data(self.chan, CryptoVec::from(s.as_bytes()))
            .map_err(|_| fmt::Error)
    }
}
