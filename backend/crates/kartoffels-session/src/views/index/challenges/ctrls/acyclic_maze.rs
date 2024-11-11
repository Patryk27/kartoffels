use super::Challenge;
use crate::views::game::{GameCtrl, HelpMsg, HelpMsgResponse, Perms};
use crate::MapBroadcaster;
use anyhow::Result;
use futures::future::BoxFuture;
use glam::{ivec2, uvec2, IVec2, UVec2};
use kartoffels_bots::DUMMY;
use kartoffels_store::Store;
use kartoffels_ui::{Msg, MsgButton, MsgLine};
use kartoffels_world::prelude::{
    BotId, Config, CreateBotRequest, Dir, Handle, Map, Policy, TileBase,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::sync::LazyLock;
use tokio::sync::mpsc;
use tracing::debug;

pub static CHALLENGE: Challenge = Challenge {
    name: "acyclic-maze",
    desc: "what can go wrong when a tiny bot enters a huge maze?",
    run,
};

static DOCS: LazyLock<Vec<MsgLine>> = LazyLock::new(|| {
    vec![
        MsgLine::new(
            "poor timmy-bot went for a walk and got attacked by a masked \
             perpetrator which took poor timmy-bot's wheels and ran away",
        ),
        MsgLine::new(""),
        MsgLine::new("*show mercy:*"),
        MsgLine::new(""),
        MsgLine::new(
            "implement a robot that traverses the maze, locates timmy and \
             _kills it_ - you'll be starting in the bottom-right corner",
        ),
    ]
});

static INIT_MSG: LazyLock<Msg<bool>> = LazyLock::new(|| Msg {
    title: Some(" acyclic-maze "),
    body: DOCS.clone(),

    buttons: vec![
        MsgButton::abort("go-back", false),
        MsgButton::confirm("start", true),
    ],
});

static HELP_MSG: LazyLock<HelpMsg> = LazyLock::new(|| Msg {
    title: Some(" help "),
    body: DOCS.clone(),
    buttons: vec![HelpMsgResponse::close()],
});

static WIN_MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" acyclic-maze "),

    body: vec![MsgLine::new(
        "congrats - poor timmy-bot is surely in a better place now, thanks to \
         you!",
    )],

    buttons: vec![MsgButton::confirm("ok", ())],
});

const AREA: UVec2 = uvec2(37, 19);
const ENTRANCE_LEN: u32 = 5;
const SIZE: UVec2 = uvec2(AREA.x + ENTRANCE_LEN, AREA.y);

const TIMMY_POS: IVec2 = ivec2(1, 1);
const PLAYER_POS: IVec2 = ivec2(35 + (ENTRANCE_LEN as i32), 17);

fn run(store: &Store, game: GameCtrl) -> BoxFuture<Result<()>> {
    Box::pin(async move {
        if !game.show_msg(&INIT_MSG).await? {
            return Ok(());
        }

        let (world, timmy) = setup(store, &game).await?;

        main(&world, timmy).await?;

        game.show_msg(&WIN_MSG).await?;

        Ok(())
    })
}

async fn setup(store: &Store, game: &GameCtrl) -> Result<(Handle, BotId)> {
    game.set_help(Some(&*HELP_MSG)).await?;
    game.set_perms(Perms::CHALLENGE.disabled()).await?;
    game.set_status(Some("building world".into())).await?;

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
        .create_bot(CreateBotRequest::new(DUMMY).at(TIMMY_POS).facing(Dir::S))
        .await?;

    game.join(world.clone()).await?;

    // ---

    MapBroadcaster::new(|tx| async move {
        let seed = if store.testing() {
            Default::default()
        } else {
            rand::thread_rng().gen()
        };

        let map = create_map_ex(seed, tx).await;

        Ok(map)
    })
    .run(store, &world)
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

async fn main(world: &Handle, timmy: BotId) -> Result<()> {
    let mut snapshots = world.snapshots();

    // Wait for Timmy to appear - that's required only for tests, because there
    // we don't animate the map and so it might happen that the code is so quick
    // we get here and don't see Timmy yet
    snapshots.wait_for_bot(timmy).await?;

    loop {
        if !snapshots.next().await?.bots().alive().has(timmy) {
            return Ok(());
        }
    }
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
            .join("views")
            .join("index")
            .join("challenges")
            .join("ctrls")
            .join("acyclic_maze")
            .join("tests")
            .join("map");

        let (tx, _) = mpsc::channel(1);
        let map = executor::block_on(create_map_ex(Default::default(), tx));

        Asserter::new(dir).assert("expected.txt", map.to_string());
    }
}
