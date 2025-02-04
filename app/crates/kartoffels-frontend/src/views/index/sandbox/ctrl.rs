use crate::utils;
use crate::views::game::{Config, GameCtrl, HelpMsg, HelpMsgEvent};
use anyhow::Result;
use glam::uvec2;
use kartoffels_store::Store;
use kartoffels_ui::{Msg, MsgLine};
use kartoffels_world::prelude::{wfc, Config as WorldConfig, Policy, Theme};
use std::future;
use std::sync::LazyLock;

const MAX_BOTS: usize = 16;

static HELP: LazyLock<HelpMsg> = LazyLock::new(|| Msg {
    title: Some(" help "),

    body: vec![
        MsgLine::new("welcome to the *sandbox*!"),
        MsgLine::new(""),
        MsgLine::new(
            "in here you're playing on your own, private world — it's a safe \
             place for you to play with, develop and debug bots",
        ),
        MsgLine::new(""),
        MsgLine::new(
            "i assume you already went through the tutorial — if not, feel \
             free to go back to the main menu and press [`t`]",
        ),
        MsgLine::new(""),
        MsgLine::new("# rules"),
        MsgLine::new(""),
        MsgLine::new(format!("- there's a limit of {MAX_BOTS} bots")),
        MsgLine::new("- you're allowed to delete bots and restart them"),
        MsgLine::new(
            "- you've got some extra commands at hand, like `spawn-bot`",
        ),
        MsgLine::new(
            "- a new world is generated every time you open the sandbox",
        ),
    ],

    buttons: vec![HelpMsgEvent::close()],
});

const CONFIG: Config = Config {
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
};

pub async fn run(store: &Store, theme: Theme, game: GameCtrl) -> Result<()> {
    init(store, theme, &game).await?;

    game.set_config(CONFIG).await?;
    game.set_status(None).await?;

    future::pending().await
}

async fn init(store: &Store, _theme: Theme, game: &GameCtrl) -> Result<()> {
    game.set_help(Some(&*HELP)).await?;
    game.set_config(CONFIG.disabled()).await?;
    game.set_status(Some("building".into())).await?;

    let world = store.create_private_world(WorldConfig {
        name: "sandbox".into(),
        policy: Policy {
            auto_respawn: true,
            max_alive_bots: MAX_BOTS,
            max_queued_bots: MAX_BOTS,
        },
        ..Default::default()
    })?;

    game.join(world.clone()).await?;

    utils::map::build(store, &world, |mut mapb, mut rng| async move {
        wfc(&mut rng, &mut mapb, uvec2(64, 32)).await?;

        // let map = theme.create_map(&mut rng)?;
        // mapb.reveal(map, &mut rng).await;

        Ok(mapb.finish())
    })
    .await?;

    Ok(())
}
