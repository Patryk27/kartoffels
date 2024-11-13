use glam::{ivec2, uvec2};
use kartoffels_bots::{DUMMY, ROBERTO};
use kartoffels_utils::Asserter;
use kartoffels_world::prelude::*;
use std::future::Future;
use std::path::Path;
use std::sync::Arc;

#[tokio::test]
async fn smoke() {
    // Use roberto compiled in the past, so that bumping the toolchain version
    // etc. don't cause the test to go stale.
    //
    // (e.g. a newer rustc could generate more optimized code, affecting how the
    // bot behaves)
    const ROBERTO: &[u8] = include_bytes!("./acceptance/smoke/roberto.elf");

    let world = kartoffels_world::create(config());
    let mut asserter = asserter("smoke");

    for _ in 0..16 {
        world
            .create_bot(CreateBotRequest::new(ROBERTO))
            .await
            .unwrap();
    }

    world.assert(&mut asserter, "1.md").await;
    world.tick().await.unwrap();
    world.assert(&mut asserter, "2.md").await;

    for _ in 0..256 {
        world.tick().await.unwrap();
    }

    world.assert(&mut asserter, "3.md").await;
}

#[tokio::test]
async fn pause_and_resume() {
    let world = kartoffels_world::create(config());

    for _ in 0..16 {
        world
            .create_bot(CreateBotRequest::new(ROBERTO))
            .await
            .unwrap();
    }

    for _ in 0..32 {
        world.tick().await.unwrap();
    }

    let snap1 = world.snapshot().await;

    for _ in 0..32 {
        world.tick().await.unwrap();
    }

    let snap2 = world.snapshot().await;

    world.pause().await.unwrap();

    for _ in 0..32 {
        world.tick().await.unwrap();
    }

    let snap3 = world.snapshot().await;

    world.resume().await.unwrap();

    for _ in 0..32 {
        world.tick().await.unwrap();
    }

    let snap4 = world.snapshot().await;

    // ---

    assert_ne!(snap1.map(), snap2.map());
    assert_ne!(snap1.bots(), snap2.bots());

    assert_eq!(snap2.map(), snap3.map());
    assert_eq!(snap2.bots(), snap3.bots());

    assert_ne!(snap3.map(), snap4.map());
    assert_ne!(snap3.bots(), snap4.bots());
}

#[tokio::test]
async fn kill_bot() {
    let world = kartoffels_world::create(config());

    let bot1 = world
        .create_bot(CreateBotRequest::new(DUMMY))
        .await
        .unwrap();

    let bot2 = world
        .create_bot(CreateBotRequest::new(DUMMY).oneshot())
        .await
        .unwrap();

    let bot3 = world
        .create_bot(CreateBotRequest::new(DUMMY))
        .await
        .unwrap();

    world.tick().await.unwrap();
    world.tick().await.unwrap();
    world.tick().await.unwrap();

    // ---

    world.kill_bot(bot1, "some reason").await.unwrap();
    world.tick().await.unwrap();

    world.kill_bot(bot2, "some reason").await.unwrap();
    world.tick().await.unwrap();

    world.kill_bot(bot3, "some reason").await.unwrap();
    world.tick().await.unwrap();

    // ---

    let snap = world.snapshot().await;

    assert!(snap.bots().alive().by_id(bot1).is_some());
    assert!(snap.bots().alive().by_id(bot2).is_none());
    assert!(snap.bots().alive().by_id(bot3).is_some());
}

#[tokio::test]
async fn destroy_bot() {
    let world = kartoffels_world::create(config());

    let bot1 = world
        .create_bot(CreateBotRequest::new(DUMMY))
        .await
        .unwrap();

    let bot2 = world
        .create_bot(CreateBotRequest::new(DUMMY))
        .await
        .unwrap();

    let bot3 = world
        .create_bot(CreateBotRequest::new(DUMMY))
        .await
        .unwrap();

    world.tick().await.unwrap();
    world.tick().await.unwrap();
    world.tick().await.unwrap();

    let snap1 = world.snapshot().await;

    world.destroy_bot(bot2).await.unwrap();
    world.tick().await.unwrap();

    let snap2 = world.snapshot().await;

    // ---

    assert!(snap1.bots().alive().by_id(bot1).is_some());
    assert!(snap1.bots().alive().by_id(bot2).is_some());
    assert!(snap1.bots().alive().by_id(bot3).is_some());

    assert!(snap2.bots().alive().by_id(bot1).is_some());
    assert!(snap2.bots().alive().by_id(bot2).is_none());
    assert!(snap2.bots().alive().by_id(bot3).is_some());
}

#[tokio::test]
async fn set_map() {
    let world = kartoffels_world::create(config());

    assert_eq!(uvec2(0, 0), world.snapshot().await.raw_map().size());

    // ---

    world.tick().await.unwrap();

    assert_eq!(uvec2(25, 25), world.snapshot().await.raw_map().size());

    // ---

    world.set_map(Map::new(uvec2(11, 22))).await.unwrap();
    world.tick().await.unwrap();

    assert_eq!(uvec2(11, 22), world.snapshot().await.raw_map().size());

    // ---

    world.set_map(Map::new(uvec2(22, 11))).await.unwrap();
    world.tick().await.unwrap();

    assert_eq!(uvec2(22, 11), world.snapshot().await.raw_map().size());
}

#[tokio::test]
async fn set_spawn() {
    let world = kartoffels_world::create(config());

    // First bot gets spawned at a random place
    world
        .create_bot(CreateBotRequest::new(DUMMY))
        .await
        .unwrap();

    world.tick().await.unwrap();

    // Second bot gets spawned at (10,9)
    world
        .create_bot(CreateBotRequest::new(DUMMY).at(ivec2(10, 9)))
        .await
        .unwrap();

    world.tick().await.unwrap();

    // Third bot gets spawned at (15,19)
    world.set_spawn(ivec2(15, 19), Dir::W).await.unwrap();

    world
        .create_bot(CreateBotRequest::new(DUMMY))
        .await
        .unwrap();

    world.tick().await.unwrap();

    // Fourth bot gets spawned at (16,1), since specifying a bot position
    // overrides the default spawn configuration
    world
        .create_bot(CreateBotRequest::new(DUMMY).at(ivec2(16, 1)))
        .await
        .unwrap();

    world.tick().await.unwrap();

    // Fifth bot doesn't get spawned, because the spawn-point is taken
    world
        .create_bot(CreateBotRequest::new(DUMMY).at(ivec2(16, 1)))
        .await
        .unwrap();

    world.tick().await.unwrap();

    // ---

    let actual: Vec<_> = world
        .snapshots()
        .next()
        .await
        .unwrap()
        .bots()
        .alive()
        .iter_sorted_by_birth()
        .map(|bot| bot.pos)
        .collect();

    let expected =
        vec![ivec2(21, 7), ivec2(10, 9), ivec2(15, 19), ivec2(16, 1)];

    assert_eq!(expected, actual);
}

#[tokio::test]
async fn with_auto_respawn() {
    let world = kartoffels_world::create(Config {
        policy: Policy {
            auto_respawn: true,
            ..config().policy
        },
        ..config()
    });

    let bot = world
        .create_bot(CreateBotRequest::new(DUMMY))
        .await
        .unwrap();

    world.tick().await.unwrap();
    world.kill_bot(bot, "because i say so").await.unwrap();
    world.tick().await.unwrap();

    assert!(world.snapshot().await.bots().alive().by_id(bot).is_some());
}

#[tokio::test]
async fn without_auto_respawn() {
    let world = kartoffels_world::create(Config {
        policy: Policy {
            auto_respawn: false,
            ..config().policy
        },
        ..config()
    });

    let bot = world
        .create_bot(CreateBotRequest::new(DUMMY))
        .await
        .unwrap();

    world.tick().await.unwrap();
    world.kill_bot(bot, "because i say so").await.unwrap();
    world.tick().await.unwrap();

    assert!(world.snapshot().await.bots().alive().by_id(bot).is_none());
}

#[tokio::test]
async fn err_too_many_robots_queued() {
    let world = kartoffels_world::create(Config {
        policy: Policy {
            auto_respawn: true,
            max_alive_bots: 10,
            max_queued_bots: 20,
        },
        ..config()
    });

    for _ in 0..20 {
        world
            .create_bot(CreateBotRequest::new(ROBERTO))
            .await
            .unwrap();
    }

    let err = world
        .create_bot(CreateBotRequest::new(ROBERTO))
        .await
        .unwrap_err()
        .to_string();

    assert_eq!("too many robots queued, try again in a moment", err);
}

#[tokio::test]
async fn err_couldnt_parse_firmware() {
    let err = kartoffels_world::create(config())
        .create_bot(CreateBotRequest::new(&[0x00]))
        .await
        .unwrap_err();

    assert_eq!(
        "couldn't parse firmware\
         \n\
         \n\
         Caused by:\
         \n    \
         Could not read bytes in range [0x0, 0x10)",
        format!("{err:?}")
    );
}

fn config() -> Config {
    Config {
        clock: Clock::Manual { steps: 1024 },
        mode: Mode::Deathmatch(DeathmatchMode::default()),
        name: "world".into(),
        path: None,
        policy: Policy {
            auto_respawn: true,
            max_alive_bots: 16,
            max_queued_bots: 16,
        },
        rng: Some(Default::default()),
        theme: Some(Theme::Arena(ArenaTheme::new(12))),
    }
}

fn asserter(test: &str) -> Asserter {
    Asserter::new(Path::new("tests").join("acceptance").join(test))
}

trait HandleExt {
    fn snapshot(&self) -> impl Future<Output = Arc<Snapshot>>;

    fn assert(
        &self,
        asserter: &mut Asserter,
        file: &str,
    ) -> impl Future<Output = ()>;
}

impl HandleExt for Handle {
    async fn snapshot(&self) -> Arc<Snapshot> {
        self.snapshots().next().await.unwrap()
    }

    async fn assert(&self, asserter: &mut Asserter, file: &str) {
        let actual = self.snapshot().await.to_string();
        let actual = format!("{}\n", actual.trim_end());

        asserter.assert(file, actual);
    }
}
