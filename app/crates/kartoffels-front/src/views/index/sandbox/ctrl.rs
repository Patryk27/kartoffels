use crate::views::game::{Config, GameCtrl, HelpMsg, HelpMsgEvent};
use crate::{Msg, utils};
use anyhow::Result;
use kartoffels_store::Store;
use kartoffels_world::prelude as w;
use std::future;
use std::sync::LazyLock;

const MAX_BOTS: u8 = 16;

static HELP: LazyLock<HelpMsg> = LazyLock::new(|| {
    Msg::new("help")
        .line("welcome to the *sandbox*!")
        .line("")
        .line(
            "in here you're playing on your own, private world - it's a safe \
             place for you to play with, develop and debug bots",
        )
        .line("")
        .line(
            "i assume you already went through the tutorial - if not, feel \
             free to go back to the main menu and press [`t`]",
        )
        .line("")
        .line("# rules")
        .line("")
        .line(format!("- there's a limit of {MAX_BOTS} bots"))
        .line("- you're allowed to delete bots and restart them")
        .line("- you've got some extra commands at hand, like `spawn-bot`")
        .line("- a new world is generated every time you open the sandbox")
        .btn(HelpMsgEvent::close_btn())
        .build()
});

const CONFIG: Config = Config {
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
};

pub async fn run(store: &Store, theme: w::Theme, game: GameCtrl) -> Result<()> {
    init(store, theme, &game).await?;

    game.set_config(CONFIG).await?;
    game.set_label(None).await?;

    future::pending().await
}

async fn init(store: &Store, theme: w::Theme, game: &GameCtrl) -> Result<()> {
    game.set_help(Some(&*HELP)).await?;
    game.set_config(CONFIG.disabled()).await?;
    game.set_label(Some("building".into())).await?;

    let world = store
        .create_private_world(w::Config {
            policy: w::Policy {
                allow_breakpoints: true,
                auto_respawn: true,
                max_alive_bots: MAX_BOTS,
                max_queued_bots: MAX_BOTS as u16,
            },
            ..store.world_config("sandbox")
        })
        .await?;

    game.visit(&world).await?;

    utils::map::build(store, game, &world, |mut rng, map| async move {
        theme.build(&mut rng, map).await
    })
    .await?;

    Ok(())
}
