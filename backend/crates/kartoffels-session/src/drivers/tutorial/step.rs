use crate::views::game::Perms;
use crate::DrivenGame;
use anyhow::Result;
use glam::ivec2;
use kartoffels_store::Store;
use kartoffels_world::prelude::{
    ArenaTheme, BotId, Config, Event, EventStreamExt, Handle, Policy,
    SnapshotStreamExt, Theme,
};
use std::task::Poll;

#[derive(Debug)]
pub struct StepCtxt {
    pub game: DrivenGame,
    pub world: Handle,
}

impl StepCtxt {
    pub async fn new(store: &Store, game: DrivenGame) -> Result<Self> {
        game.set_perms(Perms::TUTORIAL).await?;

        let world = store.create_private_world(Config {
            name: "tutorial".into(),
            policy: Policy {
                auto_respawn: false,
                max_alive_bots: 16,
                max_queued_bots: 16,
            },
            theme: Some(Theme::Arena(ArenaTheme::new(12))),
            rng: if store.is_testing() {
                Some(Default::default())
            } else {
                None
            },
            ..Default::default()
        })?;

        world.set_spawn(ivec2(12, 12), None).await?;
        game.join(world.clone()).await?;

        Ok(Self { game, world })
    }

    pub async fn wait_until_bot_is_spawned(&self) -> Result<BotId> {
        let id = {
            let mut events = self.world.events();

            loop {
                let event = events.next_or_err().await?;

                if let Event::BotCreated { id } = &*event {
                    break *id;
                }
            }
        };

        self.game
            .poll(move |ctxt| {
                if ctxt.world.bots().alive().by_id(id).is_some() {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await?;

        Ok(id)
    }

    pub async fn wait_until_bot_is_killed(&self) -> Result<()> {
        let mut events = self.world.events();

        loop {
            if let Event::BotKilled { .. } = &*events.next_or_err().await? {
                return Ok(());
            }
        }
    }

    pub async fn destroy_bots(&self) -> Result<()> {
        let snapshot = self.world.snapshots().next_or_err().await?;

        for bot in snapshot.bots().alive().iter() {
            self.world.destroy_bot(bot.id).await?;
        }

        self.game
            .poll(|ctxt| {
                if ctxt.world.bots().is_empty() {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await?;

        Ok(())
    }
}
