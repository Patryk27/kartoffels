use super::Challenge;
use crate::views::game::{GameCtrl, HelpMsg, HelpMsgResponse, Perms};
use anyhow::Result;
use futures::future::BoxFuture;
use indoc::indoc;
use kartoffels_bots::{CHL_HOSTAGE_SITUATION_GUARD, DUMMY};
use kartoffels_store::Store;
use kartoffels_ui::{Msg, MsgButton, MsgLine};
use kartoffels_world::prelude::{
    BotId, Config, CreateBotRequest, Dir, Handle, Map, Policy, TileBase,
};
use std::sync::LazyLock;
use tracing::debug;

pub static CHALLENGE: Challenge = Challenge {
    name: "hostage-situation",
    desc: "TODO",
    run,
};

static DOCS: LazyLock<Vec<MsgLine>> =
    LazyLock::new(|| vec![MsgLine::new("TODO")]);

static INIT_MSG: LazyLock<Msg<bool>> = LazyLock::new(|| Msg {
    title: Some(" hostage-situation "),
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

static _WIN_MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" hostage-situation "),
    body: vec![MsgLine::new("TODO")],
    buttons: vec![MsgButton::confirm("ok", ())],
});

fn run(store: &Store, game: GameCtrl) -> BoxFuture<Result<()>> {
    debug!("run()");

    Box::pin(async move {
        if !game.show_msg(&INIT_MSG).await? {
            return Ok(());
        }

        let (world, _hostage, _guards) = setup(store, &game).await?;

        game.join(world).await?;

        std::future::pending().await
    })
}

async fn setup(
    store: &Store,
    game: &GameCtrl,
) -> Result<(Handle, BotId, Vec<BotId>)> {
    game.set_help(Some(&*HELP_MSG)).await?;
    game.set_perms(Perms::CHALLENGE).await?;

    let world = store.create_private_world(Config {
        name: "challenge:hostage-situation".into(),
        policy: Policy {
            auto_respawn: false,
            max_alive_bots: 16,
            max_queued_bots: 16,
        },
        ..Default::default()
    })?;

    let (mut map, anchors) = Map::parse(indoc! {r#"
                  ---------
                  |.......|----------|
                  |..................|
                  |...d...|---------.|
                  |..cbe..|        |.|
       |----------|...f...|        |.|
       |a.................|        |.|
       |----------|.......|        |g|
                  ---------        |.|
    "#});

    anchors.fill(&mut map, TileBase::FLOOR);

    world.set_map(map).await?;
    world.set_spawn(anchors.get('a'), Dir::W).await?;

    let hostage = world
        .create_bot(
            CreateBotRequest::new(DUMMY)
                .at(anchors.get('b'))
                .spawn_at_once(),
        )
        .await?;

    let guards = [
        (anchors.get('c'), Dir::N),
        (anchors.get('d'), Dir::E),
        (anchors.get('e'), Dir::S),
        (anchors.get('f'), Dir::W),
    ];

    let guards = world
        .create_bots(guards.into_iter().map(|(pos, dir)| {
            CreateBotRequest::new(CHL_HOSTAGE_SITUATION_GUARD)
                .at(pos)
                .facing(dir)
                .spawn_at_once()
        }))
        .await?;

    Ok((world, hostage, guards))
}
