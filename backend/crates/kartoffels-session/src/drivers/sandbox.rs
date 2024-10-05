use crate::drivers::prelude::*;

const SIZE: UVec2 = uvec2(64, 32);
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
    let world = init(store, &game).await?;

    create_map(&world).await?;

    game.set_perms(Perms::SANDBOX).await?;
    game.set_status(None).await?;

    future::pending().await
}

async fn init(store: &Store, game: &DrivenGame) -> Result<Handle> {
    game.set_help(Some(&*HELP)).await?;
    game.set_perms(Perms::PENDING).await?;
    game.set_status(Some("BUILDING WORLD".into())).await?;

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
        theme: None,
    })?;

    game.join(world.clone()).await?;

    Ok(world)
}

async fn create_map(world: &Handle) -> Result<()> {
    utils::create_map(world, |tx| async move {
        let rng = ChaCha8Rng::from_seed(rand::thread_rng().gen());
        let map = create_map_ex(rng, tx).await?;

        Ok(map)
    })
    .await?;

    Ok(())
}

async fn create_map_ex(
    mut rng: impl RngCore,
    progress: mpsc::Sender<Map>,
) -> Result<Map> {
    const NOT_VISITED: u8 = 0;
    const VISITED: u8 = 1;

    let mut map = DungeonTheme::new(SIZE).create_map(&mut rng)?;
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

            if nth % 32 == 0 {
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
