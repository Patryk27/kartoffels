mod get_me_out_of_here;

mod prelude {
    pub(super) use crate::drivers::prelude::*;
    pub(super) use crate::game::Permissions;
    pub(super) use crate::DrivenGame;
    pub(super) use futures_util::future::BoxFuture;
    pub(super) use kartoffels_store::Store;
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

pub static CHALLENGES: &[&Challenge] = &[&get_me_out_of_here::CHALLENGE];
