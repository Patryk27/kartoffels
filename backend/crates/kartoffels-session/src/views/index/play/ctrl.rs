use crate::views::game::{GameCtrl, HelpMsg, HelpMsgResponse};
use anyhow::Result;
use kartoffels_ui::{Msg, MsgButton, MsgLine};
use kartoffels_world::prelude::Handle;
use std::future;
use std::sync::LazyLock;
use termwiz::input::KeyCode;

const CMD: &str = "git clone https://github.com/Patryk27/kartoffel";

static HELP: LazyLock<HelpMsg> = LazyLock::new(|| Msg {
    title: Some(" help "),

    body: vec![
        MsgLine::new(
            "welcome to the *online mode*! -- in here you're playing against \
             bots programmed by other people, deathmatch-style",
        ),
        MsgLine::new(""),
        MsgLine::new(
            "if you want to learn how to create your own bot, go back to the \
             main menu and press [`t`] - but here's the spirit:",
        ),
        MsgLine::new(""),
        MsgLine::new("# rules"),
        MsgLine::new(""),
        MsgLine::new("- your bot gets a point for each bot killed"),
        MsgLine::new("- unless the upload queue is full, each killed bot gets"),
        MsgLine::new("  auto-respawned upon death, to keep the party going"),
        MsgLine::new(""),
        MsgLine::new("# controls"),
        MsgLine::new(""),
        MsgLine::new("- use mouse and/or keyboard"),
        MsgLine::new("- press [`u`] to upload a bot"),
        MsgLine::new(
            "- press [`w`/`a`/`s`/`d`] or arrow keys to move the camera",
        ),
        MsgLine::new("- click on any bot visible on map to join it"),
        MsgLine::new(""),
        MsgLine::new("# uploading a bot"),
        MsgLine::new(""),
        MsgLine::new(format!(
            "run `{CMD}` and consult `README.md` for further instructions",
        )),
    ],

    buttons: vec![
        MsgButton::new(
            KeyCode::Char('c'),
            "copy-command",
            HelpMsgResponse::Copy(CMD),
        ),
        HelpMsgResponse::close(),
    ],
});

pub async fn run(world: Handle, game: GameCtrl) -> Result<()> {
    game.set_help(Some(&HELP)).await?;
    game.join(world).await?;

    future::pending().await
}
