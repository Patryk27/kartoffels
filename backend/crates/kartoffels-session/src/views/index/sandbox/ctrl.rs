use crate::utils;
use crate::views::game::{Config, GameCtrl, HelpMsg, HelpMsgResponse};
use anyhow::Result;
use kartoffels_store::Store;
use kartoffels_ui::{Msg, MsgLine};
use kartoffels_world::prelude::{
    Config as WorldConfig, Dir, Map, Policy, Theme, TileKind,
};
use rand::{Rng, RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::future;
use std::sync::LazyLock;
use tokio::sync::mpsc;

const MAX_BOTS: usize = 16;

static HELP: LazyLock<HelpMsg> = LazyLock::new(|| Msg {
    title: Some(" help "),

    body: vec![
        MsgLine::new("welcome to the *sandbox*!"),
        MsgLine::new(""),
        MsgLine::new(
            "in here you're playing on your own, private world - it's a safe \
             place for you to play with, develop and debug bots",
        ),
        MsgLine::new(""),
        MsgLine::new(
            "i'm assuming you've already went through the tutorial - if not, \
             feel free to go back to the main menu and press [`t`]",
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

    buttons: vec![HelpMsgResponse::close()],
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

async fn init(store: &Store, theme: Theme, game: &GameCtrl) -> Result<()> {
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

    utils::map::reveal(store, &world, |_, tx| async move {
        let rng = ChaCha8Rng::from_seed(rand::thread_rng().gen());
        let map = create_map(rng, theme, tx).await?;

        Ok(map)
    })
    .await?;

    Ok(())
}

async fn create_map(
    mut rng: impl RngCore,
    theme: Theme,
    progress: mpsc::Sender<Map>,
) -> Result<Map> {
    const NOT_VISITED: u8 = 0;
    const VISITED: u8 = 1;

    let mut map = theme.create_map(&mut rng)?;
    let mut nth = 0;
    let mut frontier = Vec::new();

    // ---

    for _ in 0..1024 {
        if frontier.len() >= 2 {
            break;
        }

        let pos = map.sample_pos(&mut rng);

        if map.get(pos).is_floor() {
            frontier.push(pos);
        }
    }

    // ---

    while !frontier.is_empty() {
        let idx = rng.gen_range(0..frontier.len());
        let pos = frontier.swap_remove(idx);

        if map.get(pos).meta[0] == NOT_VISITED {
            map.get_mut(pos).meta[0] = VISITED;

            for dir in Dir::all() {
                if map.contains(pos + dir) {
                    frontier.push(pos + dir);
                }
            }

            if nth % 64 == 0 {
                let map = map.clone().map(|_, tile| {
                    if tile.meta[0] == NOT_VISITED {
                        TileKind::VOID.into()
                    } else {
                        tile
                    }
                });

                _ = progress.send(map).await;
            }

            nth += 1;
        }
    }

    // ---

    map.for_each_mut(|_, tile| {
        tile.meta[0] = 0;
    });

    Ok(map)
}
