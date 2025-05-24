use super::{Challenge, CONFIG};
use crate::views::game::{GameCtrl, HelpMsg, HelpMsgEvent};
use crate::{utils, Msg, MsgButton, MsgLine};
use anyhow::Result;
use futures::future::BoxFuture;
use glam::IVec2;
use indoc::indoc;
use kartoffels_prefabs::CHL_DIAMOND_HEIST_GUARD;
use kartoffels_store::{Store, World};
use kartoffels_world::prelude as w;
use ratatui::style::Stylize;
use std::ops::ControlFlow;
use std::sync::LazyLock;
use termwiz::input::KeyCode;
use tracing::info;

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
             then drive away - you'll be starting in the bottom-left corner, \
             the exit is on the right side",
        ),
        MsgLine::new(""),
        MsgLine::new(
            "do not kill any guards, we don't want no spilled oil - our intel \
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
    title: Some("diamond-heist"),
    body: DOCS.clone(),

    buttons: vec![
        MsgButton::escape("exit", false),
        MsgButton::enter("start", true),
    ],
});

static HELP_MSG: LazyLock<HelpMsg> = LazyLock::new(|| Msg {
    title: Some("help"),
    body: DOCS.clone(),
    buttons: vec![HelpMsgEvent::close()],
});

static GUARD_KILLED_MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some("diamond-heist"),
    body: vec![MsgLine::new(
        "ayy, you killed a guard, alarming the entire facility - i told you: \
         *spill no oil!*",
    )],
    buttons: vec![MsgButton::enter("ok", ())],
});

static PLAYER_DIED_MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some("diamond-heist"),
    body: vec![MsgLine::new("ayy, you've died!")],
    buttons: vec![MsgButton::enter("ok", ())],
});

static CONGRATS_MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some("diamond-heist"),
    body: vec![
        MsgLine::new("congrats!"),
        MsgLine::new(""),
        MsgLine::new("now give me *my* diamond back and go away"),
    ],
    buttons: vec![MsgButton::enter("ok", ())],
});

fn run(store: &Store, game: GameCtrl) -> BoxFuture<Result<()>> {
    info!("run()");

    Box::pin(async move {
        let msg = game.msg_ex(&START_MSG).await?;

        if *msg.answer() {
            msg.close().await?;
        } else {
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
        game.msg_ex(&CONGRATS_MSG).await?;

        Ok(())
    })
}

async fn init(store: &Store, game: &GameCtrl) -> Result<(World, IVec2)> {
    game.set_help(Some(&*HELP_MSG)).await?;
    game.set_config(CONFIG).await?;
    game.set_label(Some("building".into())).await?;

    let world = store
        .create_private_world(w::Config {
            policy: w::Policy {
                allow_breakpoints: true,
                auto_respawn: false,
                max_alive_bots: 16,
                max_queued_bots: 16,
            },
            ..store.world_config("challenge:diamond-heist")
        })
        .await?;

    game.visit(&world).await?;

    // ---

    let (mut map, anchors) = w::Map::parse(indoc! {r#"
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

    anchors.fill(&mut map, w::TileKind::FLOOR);

    world.set_spawn(anchors.get('a'), w::AbsDir::E).await?;

    world
        .create_object(w::Object::new(w::ObjectKind::GEM), anchors.get('b'))
        .await?;

    utils::map::build(store, game, &world, |mut rng, mut mapb| async move {
        mapb.reveal(&mut rng, map).await;

        Ok(mapb.commit())
    })
    .await?;

    // ---

    let guards = [
        (anchors.get('c'), w::AbsDir::N),
        (anchors.get('d'), w::AbsDir::E),
        (anchors.get('e'), w::AbsDir::S),
        (anchors.get('f'), w::AbsDir::W),
    ];

    for (pos, dir) in guards {
        world
            .create_bot(
                w::CreateBotRequest::new(CHL_DIAMOND_HEIST_GUARD)
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
    world: &w::Handle,
    finish: IVec2,
) -> Result<ControlFlow<()>> {
    let mut events = world.events()?;

    game.sync(world.version()).await?;
    game.set_label(None).await?;
    events.sync(world.version()).await?;

    let player = events.next_born_bot().await?;

    loop {
        match events.next().await?.event {
            w::Event::BotDied { id } => {
                if id == player {
                    game.msg(&PLAYER_DIED_MSG).await?;
                } else {
                    game.msg(&GUARD_KILLED_MSG).await?;
                }

                return Ok(ControlFlow::Continue(()));
            }

            w::Event::BotMoved { id, at } => {
                if id == player && at == finish {
                    return Ok(ControlFlow::Break(()));
                }
            }

            _ => (),
        }
    }
}
