use super::Challenge;
use crate::drivers::prelude::*;
use glam::IVec2;
use tracing::debug;

pub static CHALLENGE: Challenge = Challenge {
    name: "acyclic-maze",
    desc: "your bot got lost and it's visibly distressed, help it escape!",
    run,
};

static HELP: LazyLock<HelpDialog> = LazyLock::new(|| Dialog {
    title: Some(" help "),

    body: vec![
        DialogLine::new(
            "bots do stupid things sometimes - like in here where your bot has \
             decided it's wise to enter a foreign maze totally unprepared and \
             now it's lost",
        ),
        DialogLine::new(""),
        DialogLine::new("see how it's crying? *that's genuine*"),
        DialogLine::new(""),
        DialogLine::new(
            "implement a bot that's able to escape this maze and win a prize - \
             you see that exit at the bottom right corner? that's where your \
             bot should drive to",
        ),
        DialogLine::new(""),
        DialogLine::new("---"),
        DialogLine::new(""),
        DialogLine::new(
            "press [`D`] to destroy the bot and then [`u`] to upload another \
             one - but don't worry, even though the current bot must get \
             destroyed in order for another to get uploaded, the bot's - uhm - \
             _soul_ remains alive... metaphysically",
        ),
        DialogLine::new(""),
        DialogLine::new("btw, the [`S`] dialog might come handy here"),
    ],

    buttons: vec![DialogButton::new(
        KeyCode::Escape,
        "close",
        HelpDialogResponse::Close,
    )
    .right_aligned()],
});

const SIZE: UVec2 = uvec2(37, 19);
const SPAWN: IVec2 = ivec2(1, 1);

fn run(store: &Store, game: DrivenGame) -> BoxFuture<Result<()>> {
    Box::pin(async move {
        let world = init(store, &game).await?;

        create_map(store, &world).await?;

        game.open_help().await?;
        game.set_perms(Perms::CHALLENGE).await?;
        game.set_status(None).await?;
        game.unlock().await?;

        future::pending::<()>().await;

        Ok(())
    })
}

async fn init(store: &Store, game: &DrivenGame) -> Result<Handle> {
    game.set_help(Some(&*HELP)).await?;
    game.set_perms(Perms::PENDING).await?;
    game.set_status(Some("BUILDING WORLD".into())).await?;
    game.lock().await?;

    let world = store.create_world(Config {
        clock: Default::default(),
        mode: Mode::Deathmatch(DeathmatchMode::default()),
        name: "challenge:acyclic-maze".into(),
        path: Default::default(),
        policy: Policy {
            auto_respawn: true,
            max_alive_bots: 1,
            max_queued_bots: 1,
        },
        rng: None,
        theme: None,
    })?;

    world
        .set_map({
            let mut map = Map::new(SIZE);

            map.set(SPAWN, TileBase::FLOOR);
            map
        })
        .await?;

    world.set_spawn(SPAWN, None).await?;

    let id = world.create_bot(BOT_DUMMY, None).await?;

    game.join(world.clone()).await?;
    game.join_bot(id).await?;

    Ok(world)
}

async fn create_map(store: &Store, world: &Handle) -> Result<()> {
    utils::create_map(store, world, |tx| async move {
        let seed = if store.testing {
            Default::default()
        } else {
            rand::thread_rng().gen()
        };

        let map = create_map_ex(seed, tx).await;

        Ok(map)
    })
    .await?;

    Ok(())
}

async fn create_map_ex(seed: [u8; 32], progress: mpsc::Sender<Map>) -> Map {
    const NOT_VISITED: u8 = 0;
    const VISITED: u8 = 1;

    debug!(?seed);

    let mut map = Map::new(SIZE);
    let mut rng = ChaCha8Rng::from_seed(seed);

    // ---

    map.line(
        ivec2(0, 0),
        ivec2(map.size().x as i32 - 1, 0),
        TileBase::WALL_H,
    );
    map.line(
        ivec2(0, 1),
        ivec2(0, map.size().y as i32 - 2),
        TileBase::WALL_V,
    );
    map.line(
        ivec2(0, map.size().y as i32 - 1),
        ivec2(map.size().x as i32 - 1, map.size().y as i32 - 1),
        TileBase::WALL_H,
    );
    map.line(
        ivec2(map.size().x as i32 - 1, 1),
        ivec2(map.size().x as i32 - 1, map.size().y as i32 - 2),
        TileBase::WALL_V,
    );

    // ---

    let mut nth = 0;
    let mut frontier = Vec::new();

    for dir in Dir::shuffled(&mut rng) {
        frontier.push((SPAWN, dir));
    }

    map.get_mut(SPAWN).meta[0] = VISITED;

    while !frontier.is_empty() {
        let idx = rng.gen_range(0..frontier.len());
        let (src_pos, dir) = frontier.swap_remove(idx);
        let mid_pos = src_pos + dir;
        let dst_pos = mid_pos + dir;

        if map.get(src_pos).is_void() {
            map.get_mut(src_pos).base = TileBase::FLOOR;
            map.set_if_void(src_pos - ivec2(1, 0), TileBase::WALL_V);
            map.set_if_void(src_pos + ivec2(1, 0), TileBase::WALL_V);
            map.set_if_void(src_pos - ivec2(0, 1), TileBase::WALL_H);
            map.set_if_void(src_pos + ivec2(0, 1), TileBase::WALL_H);
        }

        if map.contains(dst_pos) && map.get(dst_pos).meta[0] == NOT_VISITED {
            map.get_mut(dst_pos).meta[0] = VISITED;
            map.set(mid_pos, TileBase::FLOOR);

            match dir {
                Dir::N | Dir::S => {
                    map.set(mid_pos - ivec2(1, 0), TileBase::WALL_V);
                    map.set(mid_pos + ivec2(1, 0), TileBase::WALL_V);
                }

                Dir::E | Dir::W => {
                    map.set(mid_pos - ivec2(0, 1), TileBase::WALL_H);
                    map.set(mid_pos + ivec2(0, 1), TileBase::WALL_H);
                }
            }

            for dir in Dir::shuffled(&mut rng) {
                frontier.push((dst_pos, dir));
            }

            if nth % 3 == 0 {
                _ = progress.send(map.clone()).await;
            }

            nth += 1;
        }
    }

    map.set(map.size().as_ivec2() - ivec2(1, 2), TileBase::FLOOR);

    map.for_each_mut(|_, tile| {
        tile.meta[0] = 0;
    });

    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor;
    use kartoffels_utils::Asserter;
    use std::path::Path;

    #[test]
    fn map() {
        let dir = Path::new("src")
            .join("drivers")
            .join("challenges")
            .join("acyclic_maze")
            .join("tests")
            .join("map");

        let (tx, _) = mpsc::channel(1);
        let map = executor::block_on(create_map_ex(Default::default(), tx));

        Asserter::new(dir).assert("expected.txt", map.to_string());
    }
}
