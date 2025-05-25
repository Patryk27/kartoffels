use super::{CONFIG, Challenge};
use crate::views::game::{GameCtrl, HelpMsg};
use crate::{Msg, MsgBtn, MsgLine, theme, utils};
use anyhow::Result;
use futures::future::BoxFuture;
use glam::{UVec2, ivec2, uvec2};
use kartoffels_store::{Store, World};
use kartoffels_world::prelude as w;
use ratatui::style::Stylize;
use std::ops::ControlFlow;
use std::sync::LazyLock;
use termwiz::input::KeyCode;
use tokio::time;
use tracing::info;

pub static CHALLENGE: Challenge = Challenge {
    name: "personal-roomba",
    desc: "who let the flags out?",
    key: KeyCode::Char('p'),
    run,
};

static DOCS: LazyLock<Vec<MsgLine>> = LazyLock::new(|| {
    vec![
        MsgLine::new("our flags - our precious, precious flags"),
        MsgLine::new("misplaced"),
        MsgLine::new("misaligned").centered(),
        MsgLine::new("not where they").right_aligned(),
        MsgLine::new("should be").right_aligned(),
        MsgLine::new(""),
        MsgLine::new("--- *tidy up* ---").centered(),
        MsgLine::new(""),
        MsgLine::new(
            "you'll be put inside a maze - a dirty maze, lots of alleys and \
             cycles in it; within the corners of that maze are four flags \
             - find them and pick 'em using the `arm_pick()` function",
        ),
        MsgLine::new(""),
        MsgLine::new("sounds challenging, does it not?"),
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
        .line(
            "flags back at their place, peace in our brainmuscle - we are \
             grateful",
        )
        .btn(MsgBtn::enter("exit", ()))
        .build()
});

const SIZE: UVec2 = uvec2(41, 21);

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
    let (world, mut flags) = init(store, &game).await?;

    while let ControlFlow::Continue(_) = watch(&game, &world).await? {
        flags = reset(store, &game, &world, Some(flags)).await?;
    }

    game.sync(world.version()).await?;
    game.msg_ex(&CONGRATS_MSG).await?;

    Ok(())
}

async fn init(
    store: &Store,
    game: &GameCtrl,
) -> Result<(World, Vec<w::ObjectId>)> {
    game.set_help(Some(&*HELP_MSG)).await?;
    game.set_config(CONFIG.disabled()).await?;
    game.set_label(Some("building".into())).await?;

    let world = store
        .create_private_world(w::Config {
            policy: w::Policy {
                allow_breakpoints: true,
                auto_respawn: false,
                max_alive_bots: 1,
                max_queued_bots: 1,
            },
            ..store.world_config("challenge:personal-roomba")
        })
        .await?;

    game.visit(&world).await?;

    // ---

    utils::map::build(store, game, &world, async |mut rng, mut map| {
        map.begin(SIZE);

        utils::map::draw_maze(&mut rng, &mut map, SIZE, SIZE.as_ivec2() / 2)
            .await;

        utils::map::draw_holes(&mut rng, &mut map, 128).await;

        Ok(map.commit())
    })
    .await?;

    let flags = reset(store, game, &world, None).await?;

    // ---

    game.sync(world.version()).await?;
    game.set_config(CONFIG).await?;
    game.set_label(None).await?;

    Ok((world, flags))
}

async fn reset(
    store: &Store,
    game: &GameCtrl,
    world: &w::Handle,
    flags: Option<Vec<w::ObjectId>>,
) -> Result<Vec<w::ObjectId>> {
    if let Some(flags) = flags {
        game.set_label(Some("building".into())).await?;

        for flag in flags {
            world.delete_object(flag).await?;
        }
    }

    let mut flags = Vec::new();

    for x in [0, SIZE.x as i32 - 1] {
        for y in [0, SIZE.y as i32 - 1] {
            if !store.testing() {
                time::sleep(theme::FRAME_TIME * 15).await;
            }

            let id = world
                .create_object(w::Object::new(w::ObjectKind::FLAG), ivec2(x, y))
                .await?;

            flags.push(id);
        }
    }

    Ok(flags)
}

async fn watch(
    game: &GameCtrl,
    world: &w::Handle,
) -> Result<ControlFlow<(), ()>> {
    let mut events = world.events()?;
    let mut flags = 4;

    game.sync(world.version()).await?;
    game.set_label(None).await?;

    loop {
        match events.next().await?.event {
            w::Event::BotDied { .. } => {
                return Ok(ControlFlow::Continue(()));
            }

            w::Event::ObjectPicked { .. } => {
                flags -= 1;

                if flags == 0 {
                    return Ok(ControlFlow::Break(()));
                }
            }

            _ => (),
        }
    }
}
