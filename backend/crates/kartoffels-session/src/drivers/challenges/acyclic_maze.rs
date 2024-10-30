use super::Challenge;
use crate::drivers::prelude::*;
use glam::IVec2;
use tracing::debug;

pub static CHALLENGE: Challenge = Challenge {
    name: "acyclic-maze",
    desc: "what can go wrong when a tiny bot enters a huge maze?",
    run,
};

static INIT_MSG: LazyLock<Dialog<bool>> = LazyLock::new(|| Dialog {
    title: Some(" acyclic-maze "),
    body: INSTRUCTION.clone(),

    buttons: vec![
        DialogButton::abort("go back", false),
        DialogButton::confirm("let's do it", true),
    ],
});

static HELP: LazyLock<HelpDialog> = LazyLock::new(|| Dialog {
    title: Some(" help "),
    body: INSTRUCTION.clone(),
    buttons: vec![HelpDialogResponse::close()],
});

static INSTRUCTION: LazyLock<Vec<DialogLine>> = LazyLock::new(|| {
    vec![
        DialogLine::new(
            "poor timmy-bot went for a walk and got attacked by a masked \
             perpetrator which took poor timmy-bot's wheels and ran away",
        ),
        DialogLine::new(""),
        DialogLine::new("*show mercy:*"),
        DialogLine::new(""),
        DialogLine::new(
            "implement a robot that traverses the maze, locates timmy and \
             _kills it_ - you'll be starting in the bottom-right corner",
        ),
    ]
});

static WIN_MSG: LazyLock<Dialog<()>> = LazyLock::new(|| Dialog {
    title: Some(" acyclic-maze "),

    body: vec![DialogLine::new(
        "congrats - poor timmy-bot is surely in a better place now, thanks to \
         you!",
    )],

    buttons: vec![DialogButton::confirm("ok", ())],
});

const AREA: UVec2 = uvec2(37, 19);
const ENTRANCE_LEN: u32 = 5;
const SIZE: UVec2 = uvec2(AREA.x + ENTRANCE_LEN, AREA.y);

const TIMMY_POS: IVec2 = ivec2(1, 1);
const PLAYER_POS: IVec2 = ivec2(35 + (ENTRANCE_LEN as i32), 17);

fn run(store: &Store, game: DrivenGame) -> BoxFuture<Result<()>> {
    Box::pin(async move {
        if !game.run_dialog(&INIT_MSG).await? {
            return Ok(());
        }

        let (world, timmy) = setup(store, &game).await?;

        wait(&world, timmy).await?;

        game.run_dialog(&WIN_MSG).await?;

        Ok(())
    })
}

async fn setup(store: &Store, game: &DrivenGame) -> Result<(Handle, BotId)> {
    game.set_help(Some(&*HELP)).await?;
    game.set_perms(Perms::CHALLENGE.disabled()).await?;
    game.set_status(Some("BUILDING WORLD".into())).await?;

    let world = store.create_private_world(Config {
        name: "challenge:acyclic-maze".into(),
        policy: Policy {
            auto_respawn: false,
            max_alive_bots: 2,
            max_queued_bots: 1,
        },
        ..Default::default()
    })?;

    world
        .set_map({
            let mut map = Map::new(SIZE);

            map.set(TIMMY_POS, TileBase::FLOOR);
            map
        })
        .await?;

    world.set_spawn(PLAYER_POS, Dir::W).await?;

    let timmy = world
        .create_bot(
            CreateBotRequest::new(BOT_DUMMY)
                .at(TIMMY_POS)
                .facing(Dir::S),
        )
        .await?;

    game.join(world.clone()).await?;

    // ---

    utils::create_map(store, &world, |tx| async move {
        let seed = if store.is_testing() {
            Default::default()
        } else {
            rand::thread_rng().gen()
        };

        let map = create_map_ex(seed, tx).await;

        Ok(map)
    })
    .await?;

    game.set_perms(Perms::CHALLENGE).await?;
    game.set_status(None).await?;

    Ok((world, timmy))
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
        frontier.push((TIMMY_POS, dir));
    }

    map.get_mut(TIMMY_POS).meta[0] = VISITED;

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

async fn wait(world: &Handle, timmy: BotId) -> Result<()> {
    let mut events = world.events();

    loop {
        if events.next_or_err().await?.is_bot_killed(timmy) {
            break;
        }
    }

    Ok(())
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
