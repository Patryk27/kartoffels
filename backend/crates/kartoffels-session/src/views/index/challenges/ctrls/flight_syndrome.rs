#![allow(unused)]

use super::Challenge;
use crate::views::game::{GameCtrl, HelpDialog, HelpDialogResponse, Perms};
use anyhow::Result;
use futures::future::BoxFuture;
use kartoffels_store::Store;
use kartoffels_ui::{Dialog, DialogButton, DialogLine};
use kartoffels_world::prelude::{
    ArenaTheme, BotId, Config, Handle, Policy, Theme,
};
use std::sync::LazyLock;

pub static CHALLENGE: Challenge = Challenge {
    name: "flight-syndrome",
    desc: "you got to let me know - should i stay or should i go?",
    run,
};

static DOCS: LazyLock<Vec<DialogLine>> = LazyLock::new(|| {
    vec![
        DialogLine::new(
            "one blink of an eye and you got surrounded by a gang of \
             mischievous bots, ready to take your battery away",
        ),
        DialogLine::new(""),
        DialogLine::new("*save yourself:*"),
        DialogLine::new(""),
        DialogLine::new(
            "implement a robot that runs away - keep yourself alive for at \
             least 30 seconds, and don't fall down",
        ),
    ]
});

static INIT_MSG: LazyLock<Dialog<bool>> = LazyLock::new(|| Dialog {
    title: Some(" flight-syndrome "),
    body: DOCS.clone(),

    buttons: vec![
        DialogButton::abort("go back", false),
        DialogButton::confirm("let's do it", true),
    ],
});

static HELP_MSG: LazyLock<HelpDialog> = LazyLock::new(|| Dialog {
    title: Some(" help "),
    body: DOCS.clone(),
    buttons: vec![HelpDialogResponse::close()],
});

static _WIN_MSG: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" flight-syndrome "),
    body: vec![DialogLine::new("congrats - you're safe now!")],
    buttons: vec![DialogButton::confirm("ok", ())],
});

fn run(store: &Store, game: GameCtrl) -> BoxFuture<Result<()>> {
    Box::pin(async move {
        if !game.run_dialog(&INIT_MSG).await? {
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

    let enemies = vec![];

    // let enemies = world
    //     .create_bots(
    //         enemies
    //             .into_iter()
    //             .map(|pos| CreateBotRequest::new(src).at(pos)),
    //     )
    //     .await?;

    Ok((world, enemies))
}
