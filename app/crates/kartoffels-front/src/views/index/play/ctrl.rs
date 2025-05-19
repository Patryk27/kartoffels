use crate::views::game::{Config, GameCtrl, HelpMsg, HelpMsgEvent};
use crate::{Msg, MsgButton, MsgLine};
use anyhow::Result;
use kartoffels_store::{Session, World};
use std::future;
use std::sync::LazyLock;
use termwiz::input::KeyCode;

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
            "copy-command",
            KeyCode::Char('c'),
            HelpMsgEvent::Copy {
                payload: CMD.to_owned(),
            },
        ),
        HelpMsgEvent::close(),
    ],
});

pub async fn run(sess: &Session, world: World, game: GameCtrl) -> Result<()> {
    game.set_help(Some(&HELP)).await?;
    game.visit(&world).await?;

    if sess.with(|sess| sess.role().is_admin()) {
        game.set_config(Config {
            enabled: true,
            hero_mode: false,
            sync_pause: true,

            can_delete_bots: true,
            can_join_bots: true,
            can_overclock: false,
            can_pause: true,
            can_restart_bots: true,
            can_spawn_bots: true,
            can_upload_bots: true,
        })
        .await?;
    }

    future::pending().await
}
