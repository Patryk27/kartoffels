use crate::play::Policy;
use crate::DrivenGame;
use anyhow::Result;
use std::future;

pub async fn run(game: DrivenGame) -> Result<()> {
    game.set_policy(Policy {
        can_pause_world: true,
        can_configure_world: true,
        can_manage_bots: true,
        pause_is_propagated: true,
    })
    .await?;

    future::pending().await
}
