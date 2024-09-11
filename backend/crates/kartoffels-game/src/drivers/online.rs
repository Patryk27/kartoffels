use crate::views::play::{HelpDialog, HelpDialogResponse};
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
        DialogLine::new("hey there and welcome to kartoffels 🫡"),
        DialogLine::new(""),
        DialogLine::new(
            "you're in the *online mode*, which means that you're playing \
             against bots programmed by other people, deathmatch-style",
        ),
        DialogLine::new(""),
        DialogLine::new(
            "if you want to learn how to create your own bot, go back to the \
             main menu and press [`t`] - but here's the spirit:"
        ),
        DialogLine::new(""),
        DialogLine::new("# rules"),
        DialogLine::new(""),
        DialogLine::new("- your bot gets a point for each bot killed"),
        DialogLine::new("- unless the upload queue is full, each killed bot gets"),
        DialogLine::new("  auto-respawned upon death, to keep the party going"),
        DialogLine::new(""),
        DialogLine::new("# controls"),
        DialogLine::new(""),
        DialogLine::new("- use mouse and/or keyboard"),
        DialogLine::new("- press [`u`] to upload a bot"),
        DialogLine::new("- press [`w`/`a`/`s`/`d`] or arrow keys to move the camera"),
        DialogLine::new("- click on any bot visible on map to join it"),
        DialogLine::new(""),
        DialogLine::new("# uploading a bot"),
        DialogLine::new(""),
        DialogLine::new(format!(
            "run `{CMD}` and consult `README.md` for further instructions",
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
    game.set_help(Some(&HELP)).await?;
    game.join(handle).await?;

    future::pending().await
}
