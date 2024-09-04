mod assets;

use crate::play::Policy;
use crate::DrivenGame;
use anyhow::{Context, Result};
use kartoffels_store::Store;
use kartoffels_world::prelude::{
    ArenaThemeConfig, Config, DeathmatchModeConfig, Event, ModeConfig,
    Policy as WorldPolicy, ThemeConfig,
};
use std::future;
use std::task::Poll;
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

    let mut events = world.events();

    game.join(world.clone()).await?;

    game.poll(|world| {
        if world.bots().alive().is_empty() {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    })
    .await?;

    game.pause().await?;

    let _bot_id = loop {
        let event = events.next().await.context("world has crashed")?;

        if let Event::BotSpawned { id } = &*event {
            break *id;
        }
    };

    assets::DIALOG_06.show(&game).await?;
    assets::DIALOG_07.show(&game).await?;

    game.update_policy(|policy| {
        policy.can_pause_world = true;
    })
    .await?;

    loop {
        let event = events.next().await.context("world has crashed")?;

        if let Event::BotKilled { .. } = &*event {
            break;
        }
    }

    game.pause().await?;

    assets::DIALOG_08.show(&game).await?;

    loop {
        let event = events.next().await.context("world has crashed")?;

        if let Event::BotKilled { .. } = &*event {
            break;
        }
    }

    future::pending().await
}
