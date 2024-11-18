use super::{Challenge, CONFIG};
use crate::utils;
use crate::views::game::{GameCtrl, HelpMsg, HelpMsgResponse};
use anyhow::Result;
use futures::future::BoxFuture;
use glam::{ivec2, uvec2, UVec2};
use kartoffels_store::Store;
use kartoffels_ui::{theme, Msg, MsgButton, MsgLine};
use kartoffels_world::prelude::{
    Config, Handle, Object, ObjectId, ObjectKind, Policy,
};
use ratatui::style::Stylize;
use std::ops::ControlFlow;
use std::sync::LazyLock;
use termwiz::input::KeyCode;
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

    body: vec![],

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

        loop {
            match watch(&world).await? {
                ControlFlow::Continue(_) => {
                    game.set_status(Some("building".into())).await?;

                    flags = reset(&world, Some(flags)).await?;

                    game.set_status(None).await?;
                }

                ControlFlow::Break(_) => {
                    break;
                }
            }
        }

        game.pause().await?;
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
        name: "challenge:personal-roomba".into(),
        policy: Policy {
            auto_respawn: false,
            max_alive_bots: 1,
            max_queued_bots: 1,
        },
        ..Default::default()
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

    let flags = reset(&world, None).await?;

    // ---

    game.set_config(CONFIG).await?;
    game.set_status(None).await?;

    Ok((world, flags))
}

async fn reset(
    world: &Handle,
    flags: Option<Vec<ObjectId>>,
) -> Result<Vec<ObjectId>> {
    if let Some(flags) = flags {
        for flag in flags {
            world.delete_object(flag).await?;
        }
    }

    let mut flags = Vec::new();

    for x in [0, SIZE.x as i32 - 1] {
        for y in [0, SIZE.y as i32 - 1] {
            time::sleep(theme::FRAME_TIME * 15).await;

            let id = world
                .create_object(Object::new(ObjectKind::FLAG), ivec2(x, y))
                .await?;

            flags.push(id);
        }
    }

    Ok(flags)
}

async fn watch(world: &Handle) -> Result<ControlFlow<(), ()>> {
    let mut snapshots = world.snapshots();
    let player = snapshots.next_uploaded_bot().await?;

    loop {
        let snapshot = snapshots.next().await?;

        if !snapshot.bots().alive().has(player) {
            return Ok(ControlFlow::Continue(()));
        }

        if snapshot.objects().is_empty() {
            return Ok(ControlFlow::Break(()));
        }
    }
}
