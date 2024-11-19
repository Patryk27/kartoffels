use crate::views::game::{Config, GameCtrl};
use anyhow::Result;
use glam::ivec2;
use kartoffels_store::Store;
use kartoffels_world::prelude::{
    ArenaTheme, Clock, Config as WorldConfig, EventStream, Handle, Policy,
    SnapshotStream, Theme,
};

pub struct TutorialCtxt {
    pub game: GameCtrl,
    pub world: Handle,
    pub events: EventStream,
    pub snapshots: SnapshotStream,
}

impl TutorialCtxt {
    pub async fn new(store: &Store, game: GameCtrl) -> Result<Self> {
        game.set_config(Config {
            enabled: true,
            hero_mode: true,
            sync_pause: true,

            can_delete_bots: true,
            can_join_bots: false,
            can_overclock: false,
            can_pause: false,
            can_restart_bots: false,
            can_spawn_bots: false,
            can_upload_bots: true,
        })
        .await?;

        let world = store.create_private_world(WorldConfig {
            clock: Clock::Normal,
            policy: Policy {
                auto_respawn: false,
                max_alive_bots: 16,
                max_queued_bots: 16,
            },
            theme: Some(Theme::Arena(ArenaTheme::new(12))),
            ..store.world_config("tutorial")
        })?;

        world.set_spawn(ivec2(12, 12), None).await?;
        game.join(world.clone()).await?;

        Ok(Self {
            events: world.events()?,
            snapshots: world.snapshots(),
            game,
            world,
        })
    }

    pub async fn sync(&mut self) -> Result<()> {
        self.game.sync(self.world.version()).await?;

        Ok(())
    }

    pub async fn delete_bots(&mut self) -> Result<()> {
        let snapshot = self.snapshots.next().await?;

        let alive_ids: Vec<_> =
            snapshot.bots().alive().iter().map(|bot| bot.id).collect();

        let dead_ids: Vec<_> = snapshot.bots().dead().ids().collect();

        for id in alive_ids.iter().chain(&dead_ids) {
            self.world.delete_bot(*id).await?;
        }

        loop {
            let snapshot = self.snapshots.next().await?;

            if !snapshot.bots().alive().has_any_of(&alive_ids) {
                return Ok(());
            }

            if !snapshot.bots().dead().has_any_of(&dead_ids) {
                return Ok(());
            }
        }
    }
}
