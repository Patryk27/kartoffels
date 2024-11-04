mod acyclic_maze;
mod flight_syndrome;

use crate::views::game::GameCtrl;
use anyhow::Result;
use futures_util::future::BoxFuture;
use kartoffels_store::Store;

#[derive(Debug)]
pub struct Challenge {
    pub name: &'static str,
    pub desc: &'static str,
    pub run: fn(&Store, GameCtrl) -> BoxFuture<Result<()>>,
}

pub static CHALLENGES: &[&Challenge] = &[&acyclic_maze::CHALLENGE];
