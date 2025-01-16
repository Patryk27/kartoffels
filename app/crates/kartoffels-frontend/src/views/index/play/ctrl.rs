use crate::views::game::{GameCtrl, HelpMsg, HelpMsgResponse};
use anyhow::Result;
use kartoffels_ui::{KeyCode, Msg, MsgButton, MsgLine};
use kartoffels_world::prelude::Handle;
use std::future;
use std::sync::LazyLock;

const CMD: &str = "git clone https://github.com/Patryk27/kartoffel";

static HELP: LazyLock<HelpMsg> = LazyLock::new(|| Msg {
    title: Some(" help "),

    body: vec![
        MsgLine::new("welcome to the *online mode*!"),
        MsgLine::new(""),
        MsgLine::new(
            "in here you're playing against bots programmed by other people, \
             deathmatch-style",
        ),
        MsgLine::new(""),
        MsgLine::new(
            "i assume you already went through the tutorial — if not, feel \
             free to go back to the main menu and press [`t`]",
        ),
        MsgLine::new(""),
        MsgLine::new("# rules"),
        MsgLine::new(""),
        MsgLine::new("- your bot gets a point for each bot it kills"),
        MsgLine::new("- unless the upload queue is full, each bot gets"),
        MsgLine::new("  reincarnated upon death, to keep the party going"),
        MsgLine::new(
            "- pressing [`spc`] pauses only your view, not the actual game",
        ),
        MsgLine::new(""),
        MsgLine::new("# controls"),
        MsgLine::new(""),
        MsgLine::new("- use mouse and/or keyboard"),
        MsgLine::new("- press [`u`] to upload a bot"),
        MsgLine::new(
            "- press [`w`/`a`/`s`/`d`] or arrow keys to move the camera",
        ),
        MsgLine::new("- click on any bot on the map to join it"),
        MsgLine::new(""),
        MsgLine::new("# uploading a bot"),
        MsgLine::new(""),
        MsgLine::new(format!("run `{CMD}` and consult `README.md`")),
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
