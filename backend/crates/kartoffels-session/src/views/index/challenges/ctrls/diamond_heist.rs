use super::{Challenge, CONFIG};
use crate::utils;
use crate::views::game::{GameCtrl, HelpMsg, HelpMsgResponse};
use anyhow::Result;
use futures::future::BoxFuture;
use glam::IVec2;
use indoc::indoc;
use kartoffels_bots::CHL_DIAMOND_HEIST_GUARD;
use kartoffels_store::Store;
use kartoffels_ui::{KeyCode, Msg, MsgButton, MsgLine};
use kartoffels_world::prelude::{
    Config, CreateBotRequest, Dir, Event, Handle, Map, Object, ObjectKind,
    Policy, TileKind,
};
use ratatui::style::Stylize;
use std::ops::ControlFlow;
use std::sync::LazyLock;
use tracing::debug;

pub static CHALLENGE: Challenge = Challenge {
    name: "diamond-heist",
    desc: "are you brave enough to steal a diamond, mr james bot?",
    key: KeyCode::Char('d'),
    run,
};

static DOCS: LazyLock<Vec<MsgLine>> = LazyLock::new(|| {
    vec![
        MsgLine::new(
            "a precious, rare diamond has been stolen from us and put under a \
             guard watch in the nearby museum",
        ),
        MsgLine::new(""),
        MsgLine::new("*steal it back*").centered(),
        MsgLine::new(""),
        MsgLine::new(
            "go inside the room, take the diamond (using `arm_pick()`) and \
             then drive away — you'll be starting in the bottom-left corner, \
             the exit is on the right side",
        ),
        MsgLine::new(""),
        MsgLine::new(
            "do not kill any guards, we don't want no spilled oil — our intel \
             says the guards scan only see the 3x3 area around them, use it to \
             your advantage",
        ),
        MsgLine::new(""),
        MsgLine::new("difficulty: medium"),
        MsgLine::new("xoxo").italic().right_aligned(),
        MsgLine::new("the architects").italic().right_aligned(),
    ]
});

static START_MSG: LazyLock<Msg<bool>> = LazyLock::new(|| Msg {
    title: Some(" diamond-heist "),
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

static GUARD_KILLED_MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" diamond-heist "),
    body: vec![MsgLine::new(
        "ayy, you killed a guard, alarming the entire facility — i told you: \
         *spill no oil!*",
    )],
    buttons: vec![MsgButton::confirm("ok", ())],
});

static PLAYER_KILLED_MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" diamond-heist "),
    body: vec![MsgLine::new("ayy, you've died!")],
    buttons: vec![MsgButton::confirm("ok", ())],
});

static COMPLETED_MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" diamond-heist "),
    body: vec![
        MsgLine::new("congrats!"),
        MsgLine::new(""),
        MsgLine::new("now give me *my* diamond back and go away"),
    ],
    buttons: vec![MsgButton::confirm("ok", ())],
});

fn run(store: &Store, game: GameCtrl) -> BoxFuture<Result<()>> {
    debug!("run()");

    Box::pin(async move {
        if !game.msg(&START_MSG).await? {
            return Ok(());
        }

        let mut world;
        let mut finish;

        loop {
            (world, finish) = init(store, &game).await?;

            match watch(&game, &world, finish).await? {
                ControlFlow::Continue(_) => {
                    game.wait_for_restart().await?;
                }

                ControlFlow::Break(_) => break,
            }
        }

        game.sync(world.version()).await?;
        game.msg(&COMPLETED_MSG).await?;

        Ok(())
    })
}

async fn init(store: &Store, game: &GameCtrl) -> Result<(Handle, IVec2)> {
    game.set_help(Some(&*HELP_MSG)).await?;
    game.set_config(CONFIG).await?;
    game.set_status(Some("building".into())).await?;

    let world = store.create_private_world(Config {
        policy: Policy {
            auto_respawn: false,
            max_alive_bots: 16,
            max_queued_bots: 16,
        },
        ..store.world_config("challenge:diamond-heist")
    })?;

    game.join(world.clone()).await?;

    // ---

    let (mut map, anchors) = Map::parse(indoc! {r#"
                  -----------
                  |.........|-----------
                  |...................g+
                  |.........|-----------
                  |....d....|
                  |...cbe...|
                  |....f....|
       |----------|.........|
       +a...................|
       |----------|.........|
                  -----------
    "#});

    anchors.fill(&mut map, TileKind::FLOOR);

    world.set_spawn(anchors.get('a'), Dir::E).await?;

    world
        .create_object(Object::new(ObjectKind::GEM), anchors.get('b'))
        .await?;

    utils::map::build(store, &world, |mut mapb, mut rng| async move {
        mapb.reveal(map, &mut rng).await;

        Ok(mapb.finish())
    })
    .await?;

    // ---

    let guards = [
        (anchors.get('c'), Dir::N),
        (anchors.get('d'), Dir::E),
        (anchors.get('e'), Dir::S),
        (anchors.get('f'), Dir::W),
    ];

    for (pos, dir) in guards {
        world
            .create_bot(
                CreateBotRequest::new(CHL_DIAMOND_HEIST_GUARD)
                    .at(pos)
                    .facing(dir)
                    .instant(),
            )
            .await?;
    }

    // ---

    let finish = anchors.get('g');

    Ok((world, finish))
}

async fn watch(
    game: &GameCtrl,
    world: &Handle,
    finish: IVec2,
) -> Result<ControlFlow<()>> {
    let mut events = world.events()?;

    game.sync(world.version()).await?;
    game.set_status(None).await?;
    events.sync(world.version()).await?;

    let player = events.next_spawned_bot().await?;

    loop {
        match events.next().await?.event {
            Event::BotDied { id } => {
                if id == player {
                    game.msg(&PLAYER_KILLED_MSG).await?;
                } else {
                    game.msg(&GUARD_KILLED_MSG).await?;
                }

                return Ok(ControlFlow::Continue(()));
            }

            Event::BotMoved { id, at } => {
                if id == player && at == finish {
                    return Ok(ControlFlow::Break(()));
                }
            }

            _ => (),
        }
    }
}
