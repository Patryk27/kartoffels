use crate::views::game::{HelpDialog, HelpDialogResponse, Permissions};
use crate::DrivenGame;
use anyhow::Result;
use glam::uvec2;
use kartoffels_store::Store;
use kartoffels_ui::{Dialog, DialogButton, DialogLine};
use kartoffels_world::prelude::{
    Config, DeathmatchMode, DungeonTheme, Mode, Policy, Theme,
};
use std::future;
use std::sync::LazyLock;
use termwiz::input::KeyCode;

const MAX_BOTS: usize = 20;

#[rustfmt::skip]
static HELP: LazyLock<HelpDialog> = LazyLock::new(|| Dialog {
    title: Some(" help "),

    body: vec![
        DialogLine::new("hey there and welcome to kartoffels 🫡"),
        DialogLine::new(""),
        DialogLine::new(
            "you're in the *sandbox mode*, which means that you're playing on \
             your own, private world - this is meant as a safe place for you \
             to calmly play with and develop bots",
        ),
        DialogLine::new(""),
        DialogLine::new(
            "i'm assuming you've already went through the tutorial - if not, \
             feel free to go back and press [`t`]",
        ),
        DialogLine::new(""),
        DialogLine::new("# rules"),
        DialogLine::new(""),
        DialogLine::new(format!("- there's a limit of {MAX_BOTS} bots")),
        DialogLine::new("- as compared to the online play, in here you're allowed to"),
        DialogLine::new("  destroy bots, restart them etc."),
        DialogLine::new("- you can also spawn *roberto*, the built-in moderately"),
        DialogLine::new("  challenging bot"),
        DialogLine::new("- a new world is generated every time you open the sandbox"),
    ],

    buttons: vec![
        DialogButton::new(
            KeyCode::Escape,
            "close",
            HelpDialogResponse::Close,
        ).right_aligned(),
    ],
});

pub async fn run(store: &Store, game: DrivenGame) -> Result<()> {
    game.set_help(Some(&*HELP)).await?;
    game.set_perms(Permissions::SANDBOX).await?;

    let world = store.create_world(Config {
        clock: Default::default(),
        mode: Mode::Deathmatch(DeathmatchMode::default()),
        name: "sandbox".into(),
        path: Default::default(),
        policy: Policy {
            auto_respawn: true,
            max_alive_bots: MAX_BOTS,
            max_queued_bots: MAX_BOTS,
        },
        rng: None,
        theme: Theme::Dungeon(DungeonTheme::new(uvec2(64, 32))),
    });

    game.join(world).await?;

    future::pending().await
}
