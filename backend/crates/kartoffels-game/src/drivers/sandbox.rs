use crate::play::Permissions;
use crate::DrivenGame;
use anyhow::Result;
use std::future;

pub async fn run(game: DrivenGame) -> Result<()> {
    game.set_perms(Permissions {
        user_can_pause_world: true,
        user_can_configure_world: true,
        user_can_manage_bots: true,
        sync_pause_mode: true,
        single_bot_mode: false,
    })
    .await?;

    future::pending().await
}
