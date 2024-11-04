use crate::views::game::{GameCtrl, HelpDialog, HelpDialogResponse, Perms};
use crate::MapBroadcaster;
use anyhow::Result;
use kartoffels_store::Store;
use kartoffels_ui::{Dialog, DialogLine};
use kartoffels_world::prelude::{
    Config, Dir, Handle, Map, Policy, Theme, TileBase,
};
use rand::{Rng, RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::future;
use std::sync::LazyLock;
use tokio::sync::mpsc;

const MAX_BOTS: usize = 16;

static HELP: LazyLock<HelpDialog> = LazyLock::new(|| Dialog {
    title: Some(" help "),

    body: vec![
        DialogLine::new("hey there and welcome to kartoffels 🫡"),
        DialogLine::new(""),
        DialogLine::new(
            "you're in the *sandbox mode*, which means that you're playing in \
             your own, private world - this is meant as a safe place for you \
             to calmly play with, develop and debug bots",
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
        DialogLine::new(
            "- as compared to the online play, in here you're allowed to",
        ),
        DialogLine::new("  destroy bots, restart them etc."),
        DialogLine::new(
            "- you can also spawn prefabs, a couple of built-in bots",
        ),
        DialogLine::new(
            "- a new world is generated every time you open the sandbox",
        ),
    ],

    buttons: vec![HelpDialogResponse::close()],
});

pub async fn run(store: &Store, theme: Theme, game: GameCtrl) -> Result<()> {
    let world = init(store, &game).await?;

    create_map(store, theme, &world).await?;

    game.set_perms(Perms::SANDBOX).await?;
    game.set_status(None).await?;

    future::pending().await
}

async fn init(store: &Store, game: &GameCtrl) -> Result<Handle> {
    game.set_help(Some(&*HELP)).await?;
    game.set_perms(Perms::SANDBOX.disabled()).await?;
    game.set_status(Some("building world".into())).await?;

    let world = store.create_private_world(Config {
        name: "sandbox".into(),
        policy: Policy {
            auto_respawn: true,
            max_alive_bots: MAX_BOTS,
            max_queued_bots: MAX_BOTS,
        },
        ..Default::default()
    })?;

    game.join(world.clone()).await?;

    Ok(world)
}

async fn create_map(store: &Store, theme: Theme, world: &Handle) -> Result<()> {
    MapBroadcaster::new(|tx| async move {
        let rng = ChaCha8Rng::from_seed(rand::thread_rng().gen());
        let map = create_map_ex(rng, theme, tx).await?;

        Ok(map)
    })
    .run(store, world)
    .await?;

    Ok(())
}

async fn create_map_ex(
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
                        TileBase::VOID.into()
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
