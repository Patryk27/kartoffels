use crate::views::game::{GameCtrl, Perms};
use anyhow::Result;
use glam::ivec2;
use kartoffels_store::Store;
use kartoffels_world::prelude::{
    ArenaTheme, Config, Handle, Policy, SnapshotStream, Theme,
};

pub struct TutorialCtxt {
    pub game: GameCtrl,
    pub world: Handle,
    pub snapshots: SnapshotStream,
}

impl TutorialCtxt {
    pub async fn new(store: &Store, game: GameCtrl) -> Result<Self> {
        game.set_perms(Perms::TUTORIAL).await?;

        let world = store.create_private_world(Config {
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

    pub async fn wait_for_ui(&mut self) -> Result<()> {
        let latest_version = self.snapshots.next().await?.version();

        loop {
            if self.game.get_snapshot_version().await? >= latest_version {
                break;
            }
        }

        Ok(())
    }

    pub async fn destroy_bots(&mut self) -> Result<()> {
        for bot in self.snapshots.next().await?.bots().alive().iter() {
            self.world.destroy_bot(bot.id).await?;
        }

        Ok(())
    }
}
