use crate::play::Permissions;
use crate::DrivenGame;
use anyhow::Result;
use glam::uvec2;
use kartoffels_store::Store;
use kartoffels_world::prelude::{
    Config, DeathmatchModeConfig, DungeonThemeConfig, ModeConfig, Policy,
    ThemeConfig,
};
use std::future;

pub async fn run(store: &Store, game: DrivenGame) -> Result<()> {
    game.set_perms(Permissions {
        single_bot_mode: false,
        sync_pause: true,
        user_can_manage_bots: true,
        user_can_pause_world: true,
    })
    .await?;

    let world = store.create_world(Config {
        name: "sandbox".into(),
        mode: ModeConfig::Deathmatch(DeathmatchModeConfig {
            round_duration: None,
        }),
        theme: ThemeConfig::Dungeon(DungeonThemeConfig {
            size: uvec2(64, 32),
        }),
        policy: Policy {
            auto_respawn: true,
            max_alive_bots: 32,
            max_queued_bots: 16,
        },
    });

    game.join(world).await?;

    future::pending().await
}
