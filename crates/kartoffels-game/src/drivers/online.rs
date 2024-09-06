use crate::play::{HelpDialog, HelpDialogResponse};
use crate::DrivenGame;
use anyhow::Result;
use kartoffels_ui::{theme, Dialog, DialogButton, DialogLine};
use kartoffels_world::prelude::Handle as WorldHandle;
use ratatui::style::Stylize;
use ratatui::text::Span;
use std::future;
use std::sync::LazyLock;
use termwiz::input::KeyCode;

const CMD: &str = "git clone https://github.com/Patryk27/kartoffel";

#[rustfmt::skip]
static HELP: LazyLock<HelpDialog> = LazyLock::new(|| Dialog {
    title: Some(" help "),

    body: vec![
        DialogLine::raw("welcome to kartoffels 🫡"),
        DialogLine::raw(""),
        DialogLine::raw(
            "if you're into tutorials, just go back to the main menu and \
             press `t` - otherwise, here's a couple of tips:",
        ),
        DialogLine::raw(""),
        DialogLine::raw("# controls").bold(),
        DialogLine::raw(""),
        DialogLine::raw("- use mouse or keyboard"),
        DialogLine::raw("- press [w/a/s/d] to navigate the map"),
        DialogLine::raw("- press [u] to upload a bot"),
        DialogLine::raw("- click on any bot to join it"),
        DialogLine::raw(""),
        DialogLine::raw("# uploading a bot").bold(),
        DialogLine::raw(""),
        DialogLine::from_iter([
            Span::raw("run `"),
            Span::raw(CMD).fg(theme::WASHED_PINK),
            Span::raw("` and see README.md there for further instructions"),
        ]),
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
