use crate::play::Policy;
use crate::DrivenGame;
use anyhow::{Context, Result};
use kartoffels_store::Store;
use kartoffels_ui::Button;
use kartoffels_world::prelude::{
    ArenaThemeConfig, Config, DeathmatchModeConfig, ModeConfig,
    Policy as WorldPolicy, ThemeConfig,
};
use std::future;
use termwiz::input::KeyCode;
use tokio_stream::StreamExt;

pub async fn run(store: &Store, game: DrivenGame) -> Result<()> {
    game.set_policy(Policy {
        can_configure_world: false,
        can_manage_bots: false,
        propagate_pause: true,
    })
    .await?;

    let world = store.create_world(Config {
        name: "sandbox".into(),
        mode: ModeConfig::Deathmatch(DeathmatchModeConfig {
            round_duration: None,
        }),
        theme: ThemeConfig::Arena(ArenaThemeConfig { radius: 15 }),
        policy: WorldPolicy {
            max_alive_bots: 16,
            max_queued_bots: 16,
        },
    });

    game.join(world.clone()).await?;

    game.dialog(move |ui, close| {
        ui.info_dialog(32, 3, Some(" tutorial "), |ui| {
            ui.line("hey there 🫡");
            ui.space(1);

            if Button::new(KeyCode::Enter, "got it")
                .right_aligned()
                .render(ui)
                .pressed
            {
                _ = close.take().unwrap().send(());
            }
        });
    })
    .await?;

    let mut snapshots = world.listen().await?;

    loop {
        let msg = snapshots.next().await.context("world has crashed")?;

        if !msg.bots().is_empty() {
            break;
        }
    }

    game.pause(true).await?;

    game.dialog(move |ui, close| {
        ui.info_dialog(32, 3, Some(" tutorial "), |ui| {
            ui.line("nice!");
            ui.space(1);

            if Button::new(KeyCode::Enter, "got it")
                .right_aligned()
                .render(ui)
                .pressed
            {
                _ = close.take().unwrap().send(());
            }
        });
    })
    .await?;

    future::pending().await
}
