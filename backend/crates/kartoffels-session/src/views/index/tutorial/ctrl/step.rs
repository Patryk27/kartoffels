use crate::views::game::{Config, GameCtrl};
use anyhow::Result;
use glam::ivec2;
use kartoffels_store::Store;
use kartoffels_world::prelude::{
    ArenaTheme, Config as WorldConfig, Handle, Policy, SnapshotStream, Theme,
};

pub struct TutorialCtxt {
    pub game: GameCtrl,
    pub world: Handle,
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
            name: "tutorial".into(),
            policy: Policy {
                auto_respawn: false,
                max_alive_bots: 16,
                max_queued_bots: 16,
            },
            theme: Some(Theme::Arena(ArenaTheme::new(12))),
            rng: if store.testing() {
                Some(Default::default())
            } else {
                None
            },
            ..Default::default()
        })?;

        world.set_spawn(ivec2(12, 12), None).await?;
        game.join(world.clone()).await?;

        Ok(Self {
            snapshots: world.snapshots(),
            game,
            world,
        })
    }

    /// Waits for user interface to catch up with the latest snapshot of the
    /// world.
    pub async fn sync(&mut self) -> Result<()> {
        let latest_version = self.snapshots.next().await?.version();

        loop {
            if self.game.get_snapshot_version().await? >= latest_version {
                break;
            }
        }

        Ok(())
    }

    pub async fn delete_bots(&mut self) -> Result<()> {
        let snapshot = self.snapshots.next().await?;

        let alive_ids = snapshot.bots().alive().iter().map(|bot| bot.id);
        let dead_ids = snapshot.bots().dead().ids();

        for id in alive_ids.chain(dead_ids) {
            self.world.delete_bot(id).await?;
        }

        Ok(())
    }
}
