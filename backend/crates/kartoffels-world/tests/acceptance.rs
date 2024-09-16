use glam::ivec2;
use kartoffels_utils::Asserter;
use kartoffels_world::prelude::*;
use std::path::Path;
use tokio_stream::StreamExt;

#[tokio::test]
async fn acceptance() {
    let mut asserter = Asserter::new(Path::new("tests").join("acceptance"));

    let world = kartoffels_world::create(Config {
        clock: Clock::Manual { steps: 1024 },
        mode: ModeConfig::Deathmatch(DeathmatchModeConfig {
            round_duration: None,
        }),
        name: "world".into(),
        path: None,
        policy: Policy {
            auto_respawn: true,
            max_alive_bots: 16,
            max_queued_bots: 16,
        },
        rng: Some(Default::default()),
        theme: ThemeConfig::Arena(ArenaThemeConfig { radius: 12 }),
    });

    world
        .set_spawn(Some(ivec2(12, 12)), Some(Dir::Up))
        .await
        .unwrap();

    world.create_bot(BOT_ROBERTO, None).await.unwrap();

    world
        .snapshots()
        .next()
        .await
        .unwrap()
        .assert(&mut asserter, "s00.md");

    world.tick().await.unwrap();

    world
        .snapshots()
        .next()
        .await
        .unwrap()
        .assert(&mut asserter, "s01.md");

    for _ in 0..256 {
        world.tick().await.unwrap();
    }

    world
        .snapshots()
        .next()
        .await
        .unwrap()
        .assert(&mut asserter, "s02.md");

    asserter.finish();
}

trait SnapshotExt {
    fn assert(&self, asserter: &mut Asserter, file: &str);
}

impl SnapshotExt for Snapshot {
    fn assert(&self, asserter: &mut Asserter, file: &str) {
        asserter.assert(file, self.to_string());
    }
}
