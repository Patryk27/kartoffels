mod acyclic_maze;

mod prelude {
    pub(super) use super::Challenge;
    pub(super) use crate::drivers::prelude::*;
    pub(super) use crate::game::Permissions;
    pub(super) use crate::DrivenGame;
    pub(super) use futures_util::future::BoxFuture;
    pub(super) use kartoffels_store::Store;
    pub(super) use rand::RngCore;
    pub(super) use std::future;
}

use crate::DrivenGame;
use anyhow::Result;
use futures_util::future::BoxFuture;
use kartoffels_store::Store;

#[derive(Debug)]
pub struct Challenge {
    pub name: &'static str,
    pub desc: &'static str,
    pub run: fn(&Store, DrivenGame) -> BoxFuture<'_, Result<()>>,
}

pub static CHALLENGES: &[&Challenge] = &[&acyclic_maze::CHALLENGE];
