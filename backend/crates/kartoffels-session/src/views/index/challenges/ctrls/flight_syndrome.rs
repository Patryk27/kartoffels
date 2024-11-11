#![allow(unused)]

use super::Challenge;
use crate::views::game::{GameCtrl, HelpMsg, HelpMsgResponse, Perms};
use anyhow::Result;
use futures::future::BoxFuture;
use glam::ivec2;
use kartoffels_store::Store;
use kartoffels_ui::{Msg, MsgButton, MsgLine};
use kartoffels_world::prelude::{
    ArenaTheme, BotId, Config, CreateBotRequest, Handle, Policy, Theme,
};
use std::sync::LazyLock;

pub static CHALLENGE: Challenge = Challenge {
    name: "flight-syndrome",
    desc: "you got to let me know - should i stay or should i go?",
    run,
};

static DOCS: LazyLock<Vec<MsgLine>> = LazyLock::new(|| {
    vec![
        MsgLine::new(
            "one blink of an eye and you got surrounded by a gang of bots, \
             ready to take your battery away",
        ),
        MsgLine::new(""),
        MsgLine::new("*save yourself:*"),
        MsgLine::new(""),
        MsgLine::new(
            "implement a robot that runs away - keep yourself alive for at \
             least 15 seconds, and don't fall down",
        ),
    ]
});

static INIT_MSG: LazyLock<Msg<bool>> = LazyLock::new(|| Msg {
    title: Some(" flight-syndrome "),
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
    title: Some(" flight-syndrome "),
    body: vec![MsgLine::new("congrats - you're safe now!")],
    buttons: vec![MsgButton::confirm("ok", ())],
});

fn run(store: &Store, game: GameCtrl) -> BoxFuture<Result<()>> {
    Box::pin(async move {
        if !game.show_msg(&INIT_MSG).await? {
            return Ok(());
        }

        let _world = setup(store, &game).await?;

        Ok(())
    })
}

async fn setup(store: &Store, game: &GameCtrl) -> Result<(Handle, Vec<BotId>)> {
    game.set_help(Some(&*HELP_MSG)).await?;
    game.set_perms(Perms::CHALLENGE).await?;

    let world = store.create_private_world(Config {
        name: "challenge:flight-syndrome".into(),
        policy: Policy {
            auto_respawn: false,
            max_alive_bots: 9,
            max_queued_bots: 1,
        },
        theme: Some(Theme::Arena(ArenaTheme::new(16))),
        ..Default::default()
    })?;

    // let enemies = vec![ivec2(16, 16)];

    // let enemies = world
    //     .create_bots(enemies.into_iter().map(|pos| {
    //         CreateBotRequest::new(kartoffels_bots::CHL_FLIGHT_SYNDROME_ENEMY)
    //             .at(pos)
    //     }))
    //     .await?;

    let enemies = vec![];

    Ok((world, enemies))
}
