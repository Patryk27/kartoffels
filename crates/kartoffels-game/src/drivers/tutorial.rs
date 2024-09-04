mod assets;

use crate::play::Policy;
use crate::DrivenGame;
use anyhow::{Context, Result};
use kartoffels_store::Store;
use kartoffels_world::prelude::{
    ArenaThemeConfig, Config, DeathmatchModeConfig, ModeConfig,
    Policy as WorldPolicy, ThemeConfig,
};
use std::future;
use tokio_stream::StreamExt;

pub async fn run(store: &Store, game: DrivenGame) -> Result<()> {
    game.set_policy(Policy {
        can_pause_world: false,
        can_configure_world: false,
        can_manage_bots: false,
        pause_is_propagated: true,
    })
    .await?;

    assets::DIALOG_01.show(&game).await?;
    assets::DIALOG_02.show(&game).await?;
    assets::DIALOG_03.show(&game).await?;
    assets::DIALOG_04.show(&game).await?;
    assets::DIALOG_05.show(&game).await?;

    let world = store.create_world(Config {
        name: "sandbox".into(),
        mode: ModeConfig::Deathmatch(DeathmatchModeConfig {
            round_duration: None,
        }),
        theme: ThemeConfig::Arena(ArenaThemeConfig { radius: 10 }),
        policy: WorldPolicy {
            max_alive_bots: 16,
            max_queued_bots: 16,
        },
    });

    game.join(world.clone()).await?;

    let mut snapshots = world.listen().await?;

    loop {
        let msg = snapshots.next().await.context("world has crashed")?;

        if !msg.bots().is_empty() {
            break;
        }
    }

    game.pause(true).await?;

    assets::DIALOG_06.show(&game).await?;
    assets::DIALOG_07.show(&game).await?;

    game.update_policy(|policy| {
        policy.can_pause_world = true;
    })
    .await?;

    future::pending().await
}
