use crate::views::game::{Config, GameCtrl, HelpMsg, HelpMsgEvent};
use crate::{Msg, MsgBtn};
use anyhow::Result;
use kartoffels_store::{Session, World};
use std::future;
use std::sync::LazyLock;
use termwiz::input::KeyCode;

const CMD: &str = "git clone https://github.com/Patryk27/kartoffel";

static HELP: LazyLock<HelpMsg> = LazyLock::new(|| {
    Msg::new("help")
        .line("welcome to the *online mode*!")
        .line("")
        .line(
            "in here you're playing against bots programmed by other people, \
             deathmatch-style",
        )
        .line("")
        .line(
            "i assume you already went through the tutorial - if not, feel \
             free to go back to the main menu and press [`t`]",
        )
        .line("")
        .line("# rules")
        .line("")
        .line("- your bot gets a point for each bot it kills")
        .line("- unless the upload queue is full, each bot gets")
        .line("  reincarnated upon death, to keep the party going")
        .line("- pressing [`spc`] pauses only your view, not the actual game")
        .line("")
        .line("# controls")
        .line("")
        .line("- use mouse and/or keyboard")
        .line("- press [`u`] to upload a bot")
        .line("- press [`w`/`a`/`s`/`d`] or arrow keys to move the camera")
        .line("- click on any bot on the map to join it")
        .line("")
        .line("# uploading a bot")
        .line("")
        .line(format!("run `{CMD}` and consult `README.md`"))
        .btn(MsgBtn::new(
            "copy-command",
            KeyCode::Char('c'),
            HelpMsgEvent::Copy {
                payload: CMD.to_owned(),
            },
        ))
        .btn(HelpMsgEvent::close_btn())
        .build()
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
            can_kill_bots: true,
            can_overclock: false,
            can_pause: true,
            can_spawn_bots: true,
            can_upload_bots: true,
        })
        .await?;
    }

    future::pending().await
}
