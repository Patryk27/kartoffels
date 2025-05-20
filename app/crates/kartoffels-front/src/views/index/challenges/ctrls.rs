mod acyclic_maze;
mod diamond_heist;
mod personal_roomba;

use crate::views::game::{Config, GameCtrl};
use anyhow::Result;
use futures_util::future::BoxFuture;
use kartoffels_store::Store;
use termwiz::input::KeyCode;

const CONFIG: Config = Config {
    enabled: true,
    hero_mode: true,
    sync_pause: true,

    can_delete_bots: true,
    can_join_bots: false,
    can_kill_bots: false,
    can_overclock: true,
    can_pause: true,
    can_spawn_bots: false,
    can_upload_bots: true,
};

#[derive(Debug)]
pub struct Challenge {
    pub name: &'static str,
    pub desc: &'static str,
    pub key: KeyCode,
    pub run: fn(&Store, GameCtrl) -> BoxFuture<Result<()>>,
}

pub static CHALLENGES: &[&Challenge] = &[
    &acyclic_maze::CHALLENGE,
    &diamond_heist::CHALLENGE,
    &personal_roomba::CHALLENGE,
];
