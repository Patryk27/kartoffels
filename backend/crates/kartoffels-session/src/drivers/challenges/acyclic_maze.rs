use super::Challenge;
use crate::drivers::prelude::*;
use glam::IVec2;
use tracing::debug;

pub static CHALLENGE: Challenge = Challenge {
    name: "acyclic-maze",
    desc: "what can go wrong when you make a pact with the devil?",
    run,
};

static DIALOG: LazyLock<Dialog<bool>> = LazyLock::new(|| Dialog {
    title: Some(" acyclic-maze "),
    body: INSTRUCTION.clone(),

    buttons: vec![
        DialogButton::abort("go back", false),
        DialogButton::confirm("start", true),
    ],
});

static HELP: LazyLock<HelpDialog> = LazyLock::new(|| Dialog {
    title: Some(" help "),
    body: INSTRUCTION.clone(),
    buttons: vec![HelpDialogResponse::close()],
});

static INSTRUCTION: LazyLock<Vec<DialogLine>> = LazyLock::new(|| {
    vec![
        DialogLine::new("a bot who knows what it wants is most admirable"),
        DialogLine::new(""),
        DialogLine::new(
            "but sometimes a bot's reasoning system goes mental and decides \
             that what they want is a longer antenna... and they do stupid \
             things to achieve this goal",
        ),
        DialogLine::new(""),
        DialogLine::new("_like making a pact with the devil_"),
        DialogLine::new(""),
        DialogLine::new(
            "poor fella thought it's getting an upgrade, while in reality the \
             fiendish devil just took the bot's wheels and left it to rot in \
             the top-left corner of the maze",
        ),
        DialogLine::new(""),
        DialogLine::new(
            "*show mercy*: implement a robot that traverses the maze, locates \
             that other bot and kills it - you'll be starting in the \
             bottom-right corner",
        ),
    ]
});

const AREA: UVec2 = uvec2(37, 19);
const ENTRANCE_LEN: u32 = 5;
const SIZE: UVec2 = uvec2(AREA.x + ENTRANCE_LEN, AREA.y);

const TARGET: IVec2 = ivec2(1, 1);
const SPAWN: IVec2 = ivec2(35 + (ENTRANCE_LEN as i32), 17);

fn run(store: &Store, game: DrivenGame) -> BoxFuture<Result<()>> {
    Box::pin(async move {
        if !game.run_dialog(&DIALOG).await? {
            return Ok(());
        }

        let world = init(store, &game).await?;

        create_map(store, &world).await?;

        game.set_perms(Perms::CHALLENGE).await?;
        game.set_status(None).await?;

        future::pending::<()>().await;

        Ok(())
    })
}

async fn init(store: &Store, game: &DrivenGame) -> Result<Handle> {
    game.set_help(Some(&*HELP)).await?;
    game.set_perms(Perms::CHALLENGE.disabled()).await?;
    game.set_status(Some("BUILDING WORLD".into())).await?;

    let world = store.create_world(Config {
        clock: Default::default(),
        mode: Mode::Deathmatch(DeathmatchMode::default()),
        name: "challenge:acyclic-maze".into(),
        path: Default::default(),
        policy: Policy {
            auto_respawn: true,
            max_alive_bots: 2,
            max_queued_bots: 1,
        },
        rng: None,
        theme: None,
    })?;

    world
        .set_map({
            let mut map = Map::new(SIZE);

            map.set(TARGET, TileBase::FLOOR);
            map
        })
        .await?;

    world.set_spawn(SPAWN, Dir::W).await?;
    world.create_bot(BOT_DUMMY, TARGET, Dir::S).await?;

    game.join(world.clone()).await?;

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

    // Draw top border
    map.line(ivec2(0, 0), ivec2(AREA.x as i32 - 1, 0), TileBase::WALL_H);

    // Draw left border
    map.line(ivec2(0, 1), ivec2(0, AREA.y as i32 - 2), TileBase::WALL_V);

    // Draw bottom border
    map.line(
        ivec2(0, AREA.y as i32 - 1),
        ivec2(AREA.x as i32 - 1, AREA.y as i32 - 1),
        TileBase::WALL_H,
    );

    // Draw right border
    map.line(
        ivec2(AREA.x as i32 - 1, 1),
        ivec2(AREA.x as i32 - 1, AREA.y as i32 - 2),
        TileBase::WALL_V,
    );

    // ---

    let mut nth = 0;
    let mut frontier = Vec::new();

    for dir in Dir::shuffled(&mut rng) {
        frontier.push((TARGET, dir));
    }

    map.get_mut(TARGET).meta[0] = VISITED;

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

        if dst_pos.x >= 0
            && dst_pos.y >= 0
            && dst_pos.x < AREA.x as i32
            && dst_pos.y < AREA.y as i32
            && map.get(dst_pos).meta[0] == NOT_VISITED
        {
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

    map.for_each_mut(|_, tile| {
        tile.meta[0] = 0;
    });

    // ---

    // Draw entrance
    map.line(
        ivec2(AREA.x as i32 - 1, AREA.y as i32 - 2),
        ivec2(AREA.x as i32 - 1 + ENTRANCE_LEN as i32, AREA.y as i32 - 2),
        TileBase::FLOOR,
    );

    map.line(
        ivec2(AREA.x as i32 - 1, AREA.y as i32 - 3),
        ivec2(AREA.x as i32 - 1 + ENTRANCE_LEN as i32, AREA.y as i32 - 3),
        TileBase::WALL_H,
    );

    map.line(
        ivec2(AREA.x as i32 - 1, AREA.y as i32 - 1),
        ivec2(AREA.x as i32 - 1 + ENTRANCE_LEN as i32, AREA.y as i32 - 1),
        TileBase::WALL_H,
    );

    map.line(
        ivec2(AREA.x as i32 - 1 + ENTRANCE_LEN as i32, AREA.y as i32 - 3),
        ivec2(AREA.x as i32 - 1 + ENTRANCE_LEN as i32, AREA.y as i32 - 1),
        TileBase::WALL_V,
    );

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
