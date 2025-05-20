use super::*;

#[tokio::test]
async fn breakpoint() {
    let world = kartoffels_world::create(config());
    let mut events = world.events().unwrap();

    world
        .set_map(Map::new(uvec2(1, 1)).filled_with(TileKind::FLOOR))
        .await
        .unwrap();

    let bot = world
        .create_bot(CreateBotRequest::new(ACC_BREAKPOINT))
        .await
        .unwrap();

    world.tick(1000).await.unwrap();

    // ---

    assert_eq!(
        Event::BotBorn { id: bot },
        events.next().await.unwrap().event,
    );
    assert_eq!(
        Event::BotReachedBreakpoint { id: bot },
        events.next().await.unwrap().event,
    );

    pa::assert_eq!(
        vec![
            BotEvent::test("reached a breakpoint"),
            BotEvent::test("born"),
            BotEvent::test("uploaded"),
        ],
        world.snapshot().await.bots.alive.get(bot).unwrap().events(),
    );

    world.resume().await.unwrap();
    world.tick(1000).await.unwrap();

    // ---

    assert_eq!(
        Event::BotReachedBreakpoint { id: bot },
        events.next().await.unwrap().event,
    );

    let snap = world.snapshot().await;
    let bot = snap.bots.alive.get(bot).unwrap();

    pa::assert_eq!(
        vec![
            BotEvent::test("reached a breakpoint"),
            BotEvent::test("reached a breakpoint"),
            BotEvent::test("born"),
            BotEvent::test("uploaded"),
        ],
        bot.events(),
    );

    assert_eq!("one two ", bot.serial());
}

#[tokio::test]
async fn breakpoint_off() {
    let world = kartoffels_world::create(Config {
        policy: Policy {
            allow_breakpoints: false,
            ..config().policy
        },
        ..config()
    });

    world
        .set_map(Map::new(uvec2(1, 1)).filled_with(TileKind::FLOOR))
        .await
        .unwrap();

    let bot = world
        .create_bot(CreateBotRequest::new(ACC_BREAKPOINT))
        .await
        .unwrap();

    world.tick(1000).await.unwrap();

    pa::assert_eq!(
        vec![BotEvent::test("born"), BotEvent::test("uploaded")],
        world.snapshot().await.bots.alive.get(bot).unwrap().events()
    );
}

#[tokio::test]
async fn fall() {
    let world = kartoffels_world::create(config());

    world
        .set_map(Map::new(uvec2(3, 3)).filled_with(TileKind::FLOOR))
        .await
        .unwrap();

    let bot = world
        .create_bot(
            CreateBotRequest::new(ACC_FALL)
                .at(ivec2(1, 1))
                .facing(AbsDir::N)
                .oneshot(),
        )
        .await
        .unwrap();

    world.tick(25000).await.unwrap();

    let snap = world.snapshot().await;

    assert!(snap.bots.alive.is_empty());
    assert!(snap.bots.queued.is_empty());
    assert!(!snap.bots.dead.is_empty());

    let bot = snap.bots.dead.get(bot).unwrap();

    pa::assert_eq!(
        vec![
            BotEvent::test("fell into the void"),
            BotEvent::test("born"),
            BotEvent::test("uploaded"),
        ],
        bot.events(),
    );
}

#[tokio::test]
async fn irq() {
    let world = kartoffels_world::create(config());

    world
        .set_map(Map::new(uvec2(5, 5)).filled_with(TileKind::FLOOR))
        .await
        .unwrap();

    let bot = world
        .create_bot(
            CreateBotRequest::new(ACC_IRQ)
                .at(ivec2(2, 2))
                .facing(AbsDir::N),
        )
        .await
        .unwrap();

    world.tick(25000).await.unwrap();

    let snap = world.snapshot().await;
    let bot = snap.bots.alive.get(bot).unwrap();

    assert_eq!(
        "boot mtr mtr2 tmr0 tmr1 tmr2a tmr2b mtr mtr2 done",
        bot.serial()
    );

    assert_eq!(ivec2(2, 0), bot.pos);
}

#[tokio::test]
async fn panic() {
    let world = kartoffels_world::create(config());

    let bot = world
        .create_bot(CreateBotRequest::new(ACC_PANIC))
        .await
        .unwrap();

    world.tick(2000).await.unwrap();

    let snap = world.snapshot().await;
    let bot = snap.bots.alive.get(bot).unwrap();

    assert_eq!(
        "\n\
         panicked at crates/kartoffels-prefabs/src/acc-panic.rs:8:5:\n\
         whoopsie!",
        bot.serial(),
    );
}

#[tokio::test]
async fn radar() {
    let world = kartoffels_world::create(config());

    world
        .set_map(Map::new(uvec2(5, 5)).filled_with(TileKind::FLOOR))
        .await
        .unwrap();

    let bot = world
        .create_bot(
            CreateBotRequest::new(ACC_RADAR)
                .at(ivec2(0, 0))
                .facing(AbsDir::S),
        )
        .await
        .unwrap();

    world.tick(5000).await.unwrap();

    let snap = world.snapshot().await;
    let bot = snap.bots.alive.get(bot).unwrap();

    #[rustfmt::skip]
    let expected = vec![
        "...  ",
        "...  ",
        "..@  ",
        "     ",
        "     ",
        "",
    ].join("\n");

    assert_eq!(expected, bot.serial());
}

#[tokio::test]
async fn serial() {
    let world = kartoffels_world::create(config());

    let bot = world
        .create_bot(CreateBotRequest::new(ACC_SERIAL))
        .await
        .unwrap();

    world.tick(500).await.unwrap();

    let snap = world.snapshot().await;
    let bot = snap.bots.alive.get(bot).unwrap();

    assert_eq!("Hello, World!\n115\n", bot.serial());
}
