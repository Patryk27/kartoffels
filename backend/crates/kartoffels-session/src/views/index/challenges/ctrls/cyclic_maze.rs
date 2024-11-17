use super::{Challenge, CONFIG};
use crate::utils;
use crate::views::game::{GameCtrl, HelpMsg, HelpMsgResponse};
use anyhow::Result;
use futures::future::BoxFuture;
use glam::{ivec2, uvec2, IVec2, UVec2};
use kartoffels_store::Store;
use kartoffels_ui::{Msg, MsgButton, MsgLine};
use kartoffels_world::prelude::{Config, Dir, Handle, Map, MapBuilder, Policy};
use rand::RngCore;
use std::sync::LazyLock;
use termwiz::input::KeyCode;
use tracing::debug;

pub static CHALLENGE: Challenge = Challenge {
    name: "cyclic-maze",
    desc: "who let the flags out?",
    key: KeyCode::Char('c'),
    run,
};

static DOCS: LazyLock<Vec<MsgLine>> =
    LazyLock::new(|| vec![MsgLine::new("TODO")]);

static START_MSG: LazyLock<Msg<bool>> = LazyLock::new(|| Msg {
    title: Some(" cyclic-maze "),
    body: DOCS.clone(),

    buttons: vec![
        MsgButton::abort("go-back", false),
        MsgButton::confirm("start", true),
    ],
});

static HELP_MSG: LazyLock<HelpMsg> = LazyLock::new(|| Msg {
    title: Some(" help "),
    body: DOCS.clone(),
    buttons: vec![HelpMsgResponse::close()],
});

static COMPLETED_MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" cyclic-maze "),

    body: vec![
        // TODO
    ],

    buttons: vec![MsgButton::confirm("ok", ())],
});

const SIZE: UVec2 = uvec2(37, 19);
const SPAWN_POS: IVec2 = ivec2(18, 10);

fn run(store: &Store, game: GameCtrl) -> BoxFuture<Result<()>> {
    debug!("run()");

    Box::pin(async move {
        if !game.msg(&START_MSG).await? {
            return Ok(());
        }

        let world = init(store, &game).await?;

        watch(&world).await?;

        game.msg(&COMPLETED_MSG).await?;

        Ok(())
    })
}

async fn init(store: &Store, game: &GameCtrl) -> Result<Handle> {
    game.set_help(Some(&*HELP_MSG)).await?;
    game.set_config(CONFIG.disabled()).await?;
    game.set_status(Some("building".into())).await?;

    let world = store.create_private_world(Config {
        name: "challenge:cyclic-maze".into(),
        policy: Policy {
            auto_respawn: false,
            max_alive_bots: 1,
            max_queued_bots: 1,
        },
        ..Default::default()
    })?;

    world.set_spawn(SPAWN_POS, Dir::W).await?;
    game.join(world.clone()).await?;

    utils::map::build(store, &world, create_map).await?;

    game.set_config(CONFIG).await?;
    game.set_status(None).await?;

    Ok(world)
}

async fn create_map(mut map: MapBuilder, mut rng: impl RngCore) -> Result<Map> {
    map.init(SIZE);

    utils::map::draw_maze(&mut map, &mut rng, SIZE, SPAWN_POS).await;
    utils::map::draw_holes(&mut map, &mut rng, 64).await;

    Ok(map.finish())
}

async fn watch(world: &Handle) -> Result<()> {
    let _snapshots = world.snapshots();

    loop {
        tokio::task::yield_now().await;
    }
}
