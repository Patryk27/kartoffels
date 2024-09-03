use crate::DrivenGame;
use anyhow::Result;
use kartoffels_world::prelude::Handle as WorldHandle;
use std::future;

pub async fn run(handle: WorldHandle, game: DrivenGame) -> Result<()> {
    game.join(handle).await?;

    future::pending().await
}
