use super::{Challenge, CONFIG};
use crate::views::game::{GameCtrl, HelpMsg, HelpMsgResponse};
use anyhow::Result;
use futures::future::BoxFuture;
use indoc::indoc;
use kartoffels_bots::CHL_DIAMOND_HEIST_GUARD;
use kartoffels_store::Store;
use kartoffels_ui::{Msg, MsgButton, MsgLine};
use kartoffels_world::prelude::{
    BotId, Config, CreateBotRequest, Dir, Handle, Map, ObjectKind, Policy,
    TileKind,
};
use std::ops::ControlFlow;
use std::sync::LazyLock;
use termwiz::input::KeyCode;
use tracing::debug;

pub static CHALLENGE: Challenge = Challenge {
    name: "diamond-heist",
    desc: "TODO",
    key: KeyCode::Char('d'),
    run,
};

static DOCS: LazyLock<Vec<MsgLine>> =
    LazyLock::new(|| vec![MsgLine::new("TODO")]);

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
    body: vec![
        MsgLine::new(
            "ouch, you've killed a guard, alarming the entire facility!",
        ),
        MsgLine::new(""),
        MsgLine::new(
            "after closing this message, press [`esc`] to open the menu and \
             select [`restart game`] to try again",
        ),
    ],
    buttons: vec![MsgButton::confirm("ok", ())],
});

static PLAYER_KILLED_MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" diamond-heist "),
    body: vec![
        MsgLine::new("ouch, you've gotten killed!"),
        MsgLine::new(""),
        MsgLine::new(
            "after closing this message, press [`esc`] to open the menu and \
             select [`restart game`] to try again",
        ),
    ],
    buttons: vec![MsgButton::confirm("ok", ())],
});

static _WIN_MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" diamond-heist "),
    body: vec![MsgLine::new("TODO")],
    buttons: vec![MsgButton::confirm("ok", ())],
});

fn run(store: &Store, game: GameCtrl) -> BoxFuture<Result<()>> {
    debug!("run()");

    Box::pin(async move {
        if !game.msg(&START_MSG).await? {
            return Ok(());
        }

        loop {
            let (world, guards) = init(store, &game).await?;

            match watch(&game, &world, &guards).await? {
                ControlFlow::Continue(_) => {
                    game.wait_for_restart().await?;
                }

                ControlFlow::Break(_) => break,
            }
        }

        Ok(())
    })
}

async fn init(store: &Store, game: &GameCtrl) -> Result<(Handle, Vec<BotId>)> {
    game.set_help(Some(&*HELP_MSG)).await?;
    game.set_config(CONFIG).await?;

    let world = store.create_private_world(Config {
        name: "challenge:diamond-heist".into(),
        policy: Policy {
            auto_respawn: false,
            max_alive_bots: 16,
            max_queued_bots: 16,
        },
        ..Default::default()
    })?;

    game.join(world.clone()).await?;

    // ---

    let (mut map, anchors) = Map::parse(indoc! {r#"
                  ---------
                  |.......|-----------
                  |.................g+
                  |...d...|-----------
                  |..cbe..|
       |----------|...f...|
       +a.................|
       |----------|.......|
                  ---------
    "#});

    anchors.fill(&mut map, TileKind::FLOOR);

    world.set_map(map).await?;
    world.set_spawn(anchors.get('a'), Dir::E).await?;
    world.put_object(anchors.get('b'), ObjectKind::GEM).await?;

    // ---

    let guards = [
        (anchors.get('c'), Dir::N),
        (anchors.get('d'), Dir::E),
        (anchors.get('e'), Dir::S),
        (anchors.get('f'), Dir::W),
    ];

    let guards = world
        .create_bots(guards.into_iter().map(|(pos, dir)| {
            CreateBotRequest::new(CHL_DIAMOND_HEIST_GUARD)
                .at(pos)
                .facing(dir)
                .instant()
        }))
        .await?;

    Ok((world, guards))
}

async fn watch(
    game: &GameCtrl,
    world: &Handle,
    guards: &[BotId],
) -> Result<ControlFlow<()>> {
    let mut snapshots = world.snapshots();

    snapshots.wait_for_bots(guards).await?;

    let player = snapshots.next_uploaded_bot().await?;

    loop {
        let snapshot = snapshots.next().await?;

        if !snapshot.bots().alive().has_all_of(guards) {
            game.msg(&GUARD_KILLED_MSG).await?;

            return Ok(ControlFlow::Continue(()));
        }

        if !snapshot.bots().alive().has(player) {
            game.msg(&PLAYER_KILLED_MSG).await?;

            return Ok(ControlFlow::Continue(()));
        }
    }
}
