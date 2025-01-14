use super::{Challenge, CONFIG};
use crate::utils;
use crate::views::game::{GameCtrl, HelpMsg, HelpMsgResponse};
use anyhow::Result;
use futures::future::BoxFuture;
use glam::{ivec2, uvec2, UVec2};
use kartoffels_store::Store;
use kartoffels_ui::{theme, KeyCode, Msg, MsgButton, MsgLine};
use kartoffels_world::prelude::{
    Config, Event, Handle, Object, ObjectId, ObjectKind, Policy,
};
use ratatui::style::Stylize;
use std::ops::ControlFlow;
use std::sync::LazyLock;
use tokio::time;
use tracing::debug;

pub static CHALLENGE: Challenge = Challenge {
    name: "personal-roomba",
    desc: "who let the flags out?",
    key: KeyCode::Char('p'),
    run,
};

static DOCS: LazyLock<Vec<MsgLine>> = LazyLock::new(|| {
    vec![
        MsgLine::new("our flags — our precious, precious flags"),
        MsgLine::new("misplaced"),
        MsgLine::new("misaligned").centered(),
        MsgLine::new("not where they").right_aligned(),
        MsgLine::new("should be").right_aligned(),
        MsgLine::new(""),
        MsgLine::new("*tidy up*").centered(),
        MsgLine::new(""),
        MsgLine::new(
            "you'll be put inside a maze — a dirty maze, lots of alleys and \
             cycles in it; within the corners of that maze are four flags \
             — find them and pick 'em using the `arm_pick()` function",
        ),
        MsgLine::new(""),
        MsgLine::new("difficulty: much"),
        MsgLine::new("xoxo").italic().right_aligned(),
        MsgLine::new("the architects").italic().right_aligned(),
    ]
});

static START_MSG: LazyLock<Msg<bool>> = LazyLock::new(|| Msg {
    title: Some(" personal-roomba "),
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
    title: Some(" personal-roomba "),

    body: vec![
        MsgLine::new("congrats!"),
        MsgLine::new(""),
        MsgLine::new(
            "flags back at their place, peace in our brainmuscle — we are \
             grateful",
        ),
    ],

    buttons: vec![MsgButton::confirm("ok", ())],
});

const SIZE: UVec2 = uvec2(41, 21);

fn run(store: &Store, game: GameCtrl) -> BoxFuture<Result<()>> {
    debug!("run()");

    Box::pin(async move {
        if !game.msg(&START_MSG).await? {
            return Ok(());
        }

        let (world, mut flags) = init(store, &game).await?;

        while let ControlFlow::Continue(_) = watch(&game, &world).await? {
            flags = reset(store, &game, &world, Some(flags)).await?;
        }

        game.sync(world.version()).await?;
        game.msg(&COMPLETED_MSG).await?;

        Ok(())
    })
}

async fn init(
    store: &Store,
    game: &GameCtrl,
) -> Result<(Handle, Vec<ObjectId>)> {
    game.set_help(Some(&*HELP_MSG)).await?;
    game.set_config(CONFIG.disabled()).await?;
    game.set_status(Some("building".into())).await?;

    let world = store.create_private_world(Config {
        policy: Policy {
            auto_respawn: false,
            max_alive_bots: 1,
            max_queued_bots: 1,
        },
        ..store.world_config("challenge:personal-roomba")
    })?;

    game.join(world.clone()).await?;

    // ---

    utils::map::build(store, &world, |mut map, mut rng| async move {
        map.init(SIZE);

        utils::map::draw_maze(&mut map, &mut rng, SIZE, SIZE.as_ivec2() / 2)
            .await;

        utils::map::draw_holes(&mut map, &mut rng, 128).await;

        Ok(map.finish())
    })
    .await?;

    let flags = reset(store, game, &world, None).await?;

    // ---

    game.sync(world.version()).await?;
    game.set_config(CONFIG).await?;
    game.set_status(None).await?;

    Ok((world, flags))
}

async fn reset(
    store: &Store,
    game: &GameCtrl,
    world: &Handle,
    flags: Option<Vec<ObjectId>>,
) -> Result<Vec<ObjectId>> {
    if let Some(flags) = flags {
        game.set_status(Some("building".into())).await?;

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
                .create_object(Object::new(ObjectKind::FLAG), ivec2(x, y))
                .await?;

            flags.push(id);
        }
    }

    Ok(flags)
}

async fn watch(game: &GameCtrl, world: &Handle) -> Result<ControlFlow<(), ()>> {
    let mut events = world.events()?;
    let mut flags = 4;

    game.sync(world.version()).await?;
    game.set_status(None).await?;

    loop {
        match events.next().await?.event {
            Event::BotDied { .. } => {
                return Ok(ControlFlow::Continue(()));
            }

            Event::ObjectPicked { .. } => {
                flags -= 1;

                if flags == 0 {
                    return Ok(ControlFlow::Break(()));
                }
            }

            _ => (),
        }
    }
}
