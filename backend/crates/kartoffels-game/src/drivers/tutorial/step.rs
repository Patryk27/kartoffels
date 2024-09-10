use crate::play::Permissions;
use crate::DrivenGame;
use anyhow::Result;
use glam::ivec2;
use kartoffels_store::Store;
use kartoffels_ui::{theme, Dialog};
use kartoffels_world::prelude::{
    ArenaThemeConfig, BotId, Config, DeathmatchModeConfig, Event,
    EventStreamExt, Handle, ModeConfig, Policy, SnapshotStreamExt, ThemeConfig,
};
use std::task::Poll;
use tokio::sync::oneshot;
use tokio::time;

#[derive(Debug)]
pub struct StepCtxt {
    pub game: DrivenGame,
    pub world: Handle,
}

impl StepCtxt {
    pub async fn new(store: &Store, game: DrivenGame) -> Result<Self> {
        game.set_perms(Permissions {
            user_can_pause_world: false,
            user_can_configure_world: false,
            user_can_manage_bots: false,
            sync_pause_mode: true,
            single_bot_mode: true,
        })
        .await?;

        let world = store.create_world(Config {
            name: "sandbox".into(),
            mode: ModeConfig::Deathmatch(DeathmatchModeConfig {
                round_duration: None,
            }),
            theme: ThemeConfig::Arena(ArenaThemeConfig { radius: 12 }),
            policy: Policy {
                auto_respawn: false,
                max_alive_bots: 16,
                max_queued_bots: 16,
            },
        });

        world.set_spawn(Some(ivec2(12, 12)), None).await?;
        game.join(world.clone()).await?;

        Ok(Self { game, world })
    }

    pub async fn run_dialog<T>(&self, dialog: &'static Dialog<T>) -> Result<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let mut tx = Some(tx);

        self.game
            .open_dialog(move |ui| {
                if let Some(resp) = dialog.render(ui) {
                    if let Some(tx) = tx.take() {
                        _ = tx.send(resp);
                    }
                }
            })
            .await?;

        let response = rx.await?;

        time::sleep(theme::INTERACTION_TIME).await;

        self.game.close_dialog().await?;

        Ok(response)
    }

    pub async fn wait_until_bot_is_created(&self) -> Result<BotId> {
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
