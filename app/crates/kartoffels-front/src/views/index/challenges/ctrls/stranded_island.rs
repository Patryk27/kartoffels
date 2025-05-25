use super::{CONFIG, Challenge};
use crate::views::game::{GameCtrl, HelpMsg};
use crate::{Msg, MsgBtn, MsgLine, utils};
use anyhow::Result;
use futures::future::BoxFuture;
use kartoffels_store::{Store, World};
use kartoffels_world::prelude as w;
use ratatui::style::Stylize;
use std::sync::LazyLock;
use termwiz::input::KeyCode;
use tracing::info;

pub static CHALLENGE: Challenge = Challenge {
    name: "stranded-island",
    desc: "will stick and stones save your bones?",
    key: KeyCode::Char('s'),
    run,
};

static DOCS: LazyLock<Vec<MsgLine>> = LazyLock::new(|| {
    vec![
        MsgLine::new("TODO"),
        MsgLine::new(""),
        MsgLine::new("sounds tricky, does it not?"),
        MsgLine::new("xoxo").italic().right_aligned(),
        MsgLine::new("the architects").italic().right_aligned(),
    ]
});

static START_MSG: LazyLock<Msg<bool>> =
    LazyLock::new(|| Msg::start(CHALLENGE.name, &DOCS));

static HELP_MSG: LazyLock<HelpMsg> = LazyLock::new(|| Msg::help(DOCS.clone()));

static CONGRATS_MSG: LazyLock<Msg> = LazyLock::new(|| {
    Msg::new(CHALLENGE.name)
        .line("congrats!")
        .line("")
        .line("TODO")
        .btn(MsgBtn::enter("exit", ()))
        .build()
});

fn run(store: &Store, game: GameCtrl) -> BoxFuture<Result<()>> {
    info!("run()");

    Box::pin(async move {
        let msg = game.msg_ex(&START_MSG).await?;

        if *msg.answer() {
            msg.close().await?;
            main(store, game).await
        } else {
            Ok(())
        }
    })
}

async fn main(store: &Store, game: GameCtrl) -> Result<()> {
    let world = init(store, &game).await?;

    watch(&world).await?;

    game.sync(world.version()).await?;
    game.msg_ex(&CONGRATS_MSG).await?;

    Ok(())
}

async fn init(store: &Store, game: &GameCtrl) -> Result<World> {
    game.set_help(Some(&*HELP_MSG)).await?;
    game.set_config(CONFIG.disabled()).await?;
    game.set_label(Some("building".into())).await?;

    let world = store
        .create_private_world(w::Config {
            policy: w::Policy {
                allow_breakpoints: true,
                auto_respawn: false,
                max_alive_bots: 2,
                max_queued_bots: 1,
            },
            ..store.world_config("challenge:stranded-island")
        })
        .await?;

    game.visit(&world).await?;

    // ---

    utils::map::build(store, game, &world, async |mut rng, mut map| {
        map.set_env(w::TileKind::WATER);

        // TODO
        w::ArenaTheme::new(12).build(&mut rng, map).await
    })
    .await?;

    // ---

    game.sync(world.version()).await?;
    game.set_config(CONFIG).await?;
    game.set_label(None).await?;

    Ok(world)
}

async fn watch(world: &w::Handle) -> Result<()> {
    let mut events = world.events()?;

    loop {
        events.next_dropped_object().await?;

        // TODO check map
    }
}
