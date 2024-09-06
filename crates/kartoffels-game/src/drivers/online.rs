use crate::play::{HelpDialog, HelpDialogResponse};
use crate::DrivenGame;
use anyhow::Result;
use kartoffels_ui::{Dialog, DialogButton, DialogLine};
use kartoffels_world::prelude::Handle as WorldHandle;
use std::future;
use std::sync::LazyLock;
use termwiz::input::KeyCode;

const CMD: &str = "git clone https://github.com/Patryk27/kartoffel";

#[rustfmt::skip]
static HELP: LazyLock<HelpDialog> = LazyLock::new(|| Dialog {
    title: Some(" help "),

    body: vec![
        DialogLine::new("welcome to kartoffels 🫡"),
        DialogLine::new(""),
        // TODO
        // DialogLine::raw("you're in the online-play mode, which means [...]"),
        DialogLine::new(
            "if you're into tutorials, just go back to the main menu and \
             press [`t`] - otherwise, here's a couple of tips:",
        ),
        DialogLine::new(""),
        DialogLine::new("# controls"),
        DialogLine::new(""),
        DialogLine::new("- use mouse or keyboard"),
        DialogLine::new("- press [`w`/`a`/`s`/`d`] to navigate the map"),
        DialogLine::new("- press [`u`] to upload a bot"),
        DialogLine::new("- click on any bot to join it"),
        DialogLine::new(""),
        DialogLine::new("# uploading a bot"),
        DialogLine::new(""),
        DialogLine::new(format!(
            "run `{}` and consult README.md for further instructions",
            CMD,
        )),
    ],

    buttons: vec![
        DialogButton::new(
            KeyCode::Char('c'),
            "copy command",
            HelpDialogResponse::Copy(CMD),
        ),

        DialogButton::new(
            KeyCode::Escape,
            "close",
            HelpDialogResponse::Close,
        ).right_aligned(),
    ],
});

pub async fn run(handle: WorldHandle, game: DrivenGame) -> Result<()> {
    game.set_help(&HELP).await?;
    game.join(handle).await?;

    future::pending().await
}
