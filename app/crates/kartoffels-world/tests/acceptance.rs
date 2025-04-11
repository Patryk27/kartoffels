use glam::{ivec2, uvec2};
use indoc::indoc;
use kartoffels_prefabs::{DUMMY, ROBERTO};
use kartoffels_utils::{Asserter, ErrorExt};
use kartoffels_world::prelude::*;
use std::future::Future;
use std::path::Path;
use std::sync::Arc;
use tempfile::NamedTempFile;

#[tokio::test]
async fn smoke() {
    let world = kartoffels_world::create(config());
    let mut asserter = asserter("smoke");

    for _ in 0..16 {
        world
            .create_bot(CreateBotRequest::new(ROBERTO))
            .await
            .unwrap();
    }

    world.assert(&mut asserter, "1.md").await;
    world.assert_json(&mut asserter, "1.json").await;

    world.tick(256).await.unwrap();

    world.assert(&mut asserter, "2.md").await;
    world.assert_json(&mut asserter, "2.json").await;
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

    world.tick(32_000).await.unwrap();

    let snap1 = world.snapshot().await;

    world.tick(32_000).await.unwrap();

    let snap2 = world.snapshot().await;

    world.pause().await.unwrap();
    world.tick(32_000).await.unwrap();

    let snap3 = world.snapshot().await;

    world.resume().await.unwrap();
    world.tick(32_000).await.unwrap();

    let snap4 = world.snapshot().await;

    // ---

    assert_ne!(snap1.map, snap2.map);
    assert_ne!(snap1.bots, snap2.bots);

    assert_eq!(snap2.map, snap3.map);
    assert_eq!(snap2.bots, snap3.bots);

    assert_ne!(snap3.map, snap4.map);
    assert_ne!(snap3.bots, snap4.bots);
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

    // ---

    world.kill_bot(bot1, "some reason").await.unwrap();
    world.kill_bot(bot2, "some reason").await.unwrap();
    world.kill_bot(bot3, "some reason").await.unwrap();

    world.tick(1).await.unwrap();

    // ---

    let snap = world.snapshot().await;

    assert!(snap.bots.alive.get(bot1).is_some());
    assert!(snap.bots.alive.get(bot2).is_none());
    assert!(snap.bots.alive.get(bot3).is_some());
}

#[tokio::test]
async fn delete_bot() {
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

    // ---

    world.tick(1).await.unwrap();

    let snap1 = world.snapshot().await;

    world.delete_bot(bot2).await.unwrap();
    world.tick(1).await.unwrap();

    let snap2 = world.snapshot().await;

    // ---

    assert!(snap1.bots.alive.get(bot1).is_some());
    assert!(snap1.bots.alive.get(bot2).is_some());
    assert!(snap1.bots.alive.get(bot3).is_some());

    assert!(snap2.bots.alive.get(bot1).is_some());
    assert!(snap2.bots.alive.get(bot2).is_none());
    assert!(snap2.bots.alive.get(bot3).is_some());
}

#[tokio::test]
async fn set_map() {
    let world = kartoffels_world::create(config());

    assert_eq!(uvec2(0, 0), world.snapshot().await.map.size());
    assert_eq!(uvec2(0, 0), world.snapshot().await.tiles.size());

    // ---

    world.tick(1).await.unwrap();

    assert_eq!(uvec2(25, 25), world.snapshot().await.map.size());
    assert_eq!(uvec2(25, 25), world.snapshot().await.tiles.size());

    // ---

    world.set_map(Map::new(uvec2(11, 22))).await.unwrap();
    world.tick(1).await.unwrap();

    assert_eq!(uvec2(11, 22), world.snapshot().await.map.size());
    assert_eq!(uvec2(11, 22), world.snapshot().await.tiles.size());

    // ---

    world.set_map(Map::new(uvec2(22, 11))).await.unwrap();
    world.tick(1).await.unwrap();

    assert_eq!(uvec2(22, 11), world.snapshot().await.map.size());
    assert_eq!(uvec2(22, 11), world.snapshot().await.tiles.size());
}

#[tokio::test]
async fn set_spawn() {
    let world = kartoffels_world::create(config());

    // First bot gets born at a random place
    world
        .create_bot(CreateBotRequest::new(DUMMY))
        .await
        .unwrap();

    // Second bot gets born at (10,9)
    world
        .create_bot(CreateBotRequest::new(DUMMY).at(ivec2(10, 9)))
        .await
        .unwrap();

    // Third bot gets born at (15,19)
    world.set_spawn(ivec2(15, 19), Dir::W).await.unwrap();

    world
        .create_bot(CreateBotRequest::new(DUMMY))
        .await
        .unwrap();

    // Fourth bot gets born at (16,1), since specifying a bot position
    // overrides the default spawn configuration
    world
        .create_bot(CreateBotRequest::new(DUMMY).at(ivec2(16, 1)))
        .await
        .unwrap();

    // Fifth bot doesn't get born, because the spawn-point is taken
    world
        .create_bot(CreateBotRequest::new(DUMMY).at(ivec2(16, 1)))
        .await
        .unwrap();

    // ---

    let actual: Vec<_> = world
        .snapshots()
        .next()
        .await
        .unwrap()
        .bots
        .alive
        .iter_sorted_by_birth()
        .map(|bot| bot.pos)
        .collect();

    let expected =
        vec![ivec2(19, 13), ivec2(10, 9), ivec2(15, 19), ivec2(16, 1)];

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

    world.kill_bot(bot, "oopsie").await.unwrap();
    world.tick(1).await.unwrap();

    let snapshot = world.snapshot().await;

    assert!(snapshot.bots.alive.get(bot).is_some());
    assert!(snapshot.bots.dead.get(bot).is_none());
    assert!(snapshot.bots.queued.get(bot).is_none());

    let expected = vec![
        "reincarnated",
        "awaiting reincarnation",
        "oopsie",
        "born",
        "uploaded",
    ];

    let actual: Vec<_> = snapshot
        .bots
        .alive
        .get(bot)
        .unwrap()
        .events
        .iter()
        .map(|event| event.msg.clone())
        .collect();

    assert_eq!(expected, actual);
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

    world.kill_bot(bot, "oopsie").await.unwrap();
    world.tick(1).await.unwrap();

    let snapshot = world.snapshot().await;

    assert!(snapshot.bots.alive.get(bot).is_none());
    assert!(snapshot.bots.dead.get(bot).is_some());
    assert!(snapshot.bots.queued.get(bot).is_none());

    let expected = vec!["oopsie", "born", "uploaded"];

    let actual: Vec<_> = snapshot
        .bots
        .dead
        .get(bot)
        .unwrap()
        .events
        .iter()
        .map(|event| event.msg.clone())
        .collect();

    assert_eq!(expected, actual);
}

#[tokio::test]
async fn resume() {
    let file = NamedTempFile::new().unwrap();

    let world = kartoffels_world::create(Config {
        path: Some(file.path().to_owned()),
        ..config()
    });

    let bot = world
        .create_bot(CreateBotRequest::new(DUMMY))
        .await
        .unwrap();

    // ---

    world.shutdown().await.unwrap();

    assert_eq!(
        "world has crashed",
        world.pause().await.unwrap_err().to_string()
    );

    // ---

    let world = kartoffels_world::resume(world.id(), file.path()).unwrap();

    world.tick(1).await.unwrap();

    let actual: Vec<_> = world
        .snapshot()
        .await
        .bots
        .alive
        .iter()
        .map(|bot| bot.id)
        .collect();

    let expected = vec![bot];

    assert_eq!(expected, actual);
}

#[tokio::test]
async fn err_too_many_bots_queued() {
    let world = kartoffels_world::create(Config {
        policy: Policy {
            auto_respawn: true,
            max_alive_bots: 10,
            max_queued_bots: 20,
        },
        ..config()
    });

    for _ in 0..30 {
        world
            .create_bot(CreateBotRequest::new(DUMMY))
            .await
            .unwrap();
    }

    let err = world
        .create_bot(CreateBotRequest::new(DUMMY))
        .await
        .unwrap_err()
        .to_string();

    assert_eq!("too many bots queued, try again in a moment", err);
}

#[tokio::test]
async fn err_couldnt_parse_firmware_1() {
    let actual = kartoffels_world::create(config())
        .create_bot(CreateBotRequest::new(&[0x00]))
        .await
        .unwrap_err()
        .to_fmt_string();

    let expected = indoc! {"
        couldn't parse firmware

        caused by:
        could not read bytes in range [0x0, 0x10)
    "};

    assert_eq!(expected.trim_end(), actual);
}

#[tokio::test]
async fn err_couldnt_parse_firmware_2() {
    const BOT: &[u8] =
        include_bytes!("./acceptance/err-couldnt-parse-firmware-2/bot.elf");

    let actual = kartoffels_world::create(config())
        .create_bot(CreateBotRequest::new(BOT))
        .await
        .unwrap_err()
        .to_fmt_string();

    let expected = indoc! {"
        couldn't parse firmware

        caused by:
        expected a 32-bit binary, but got a 64-bit one

        this is most likely the outcome of a backwards-incompatible change \
        introduced in kartoffels v0.7 - if you're following the kartoffel \
        repository, simply clone it again and copy your code there

        sorry for the trouble and godspeed!
    "};

    assert_eq!(expected.trim_end(), actual);
}

fn config() -> Config {
    Config {
        clock: Clock::manual(),
        events: false,
        id: None,
        name: "world".into(),
        path: None,
        policy: Policy {
            auto_respawn: true,
            max_alive_bots: 16,
            max_queued_bots: 16,
        },
        seed: Some(Default::default()),
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

    fn assert_json(
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
        let actual = self.snapshot().await;
        let actual = format!("{}\n", actual.to_string().trim_end());

        asserter.assert(file, actual);
    }

    async fn assert_json(&self, asserter: &mut Asserter, file: &str) {
        let actual = self.snapshot().await;
        let actual = serde_json::to_string_pretty(&actual).unwrap();

        asserter.assert(file, actual);
    }
}
