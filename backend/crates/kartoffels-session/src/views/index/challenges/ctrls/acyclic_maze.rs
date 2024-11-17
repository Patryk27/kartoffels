use super::{Challenge, CONFIG};
use crate::utils;
use crate::views::game::{GameCtrl, HelpMsg, HelpMsgResponse};
use anyhow::Result;
use futures::future::BoxFuture;
use glam::{ivec2, uvec2, IVec2, UVec2};
use kartoffels_bots::DUMMY;
use kartoffels_store::Store;
use kartoffels_ui::{Msg, MsgButton, MsgLine};
use kartoffels_world::prelude::{
    BotId, Config, CreateBotRequest, Dir, Handle, Map, Policy, TileKind,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use ratatui::style::Stylize;
use std::sync::LazyLock;
use termwiz::input::KeyCode;
use tokio::sync::mpsc;
use tracing::debug;

pub static CHALLENGE: Challenge = Challenge {
    name: "acyclic-maze",
    desc: "will you help a friend in need?",
    key: KeyCode::Char('a'),
    run,
};

static DOCS: LazyLock<Vec<MsgLine>> = LazyLock::new(|| {
    vec![
        MsgLine::new(
            "timmy-bot went for a walk and got mugged by a perpetrator who \
             took poor timmy-bot's wheels and ran away, leaving him stranded",
        ),
        MsgLine::new(""),
        MsgLine::new("*show mercy:*"),
        MsgLine::new(""),
        MsgLine::new(
            "traverse the maze, find timmy and kill it - you'll be starting in \
             the bottom-right corner",
        ),
        MsgLine::new(""),
        MsgLine::new("xoxo").italic().right_aligned(),
        MsgLine::new("the architects").italic().right_aligned(),
    ]
});

static START_MSG: LazyLock<Msg<bool>> = LazyLock::new(|| Msg {
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

static COMPLETED_MSG: LazyLock<Msg> = LazyLock::new(|| Msg {
    title: Some(" acyclic-maze "),

    body: vec![
        MsgLine::new("congrats!"),
        MsgLine::new(""),
        MsgLine::new(
            "poor timmy-bot is surely in a better place now, thanks to you!",
        ),
    ],

    buttons: vec![MsgButton::confirm("ok", ())],
});

const AREA: UVec2 = uvec2(37, 19);
const ENTRANCE_LEN: u32 = 5;
const SIZE: UVec2 = uvec2(AREA.x + ENTRANCE_LEN, AREA.y);

const TIMMY_POS: IVec2 = ivec2(1, 1);
const SPAWN_POS: IVec2 = ivec2(35 + (ENTRANCE_LEN as i32), 17);

fn run(store: &Store, game: GameCtrl) -> BoxFuture<Result<()>> {
    debug!("run()");

    Box::pin(async move {
        if !game.msg(&START_MSG).await? {
            return Ok(());
        }

        let (world, timmy) = init(store, &game).await?;

        watch(&world, timmy).await?;

        game.msg(&COMPLETED_MSG).await?;

        Ok(())
    })
}

async fn init(store: &Store, game: &GameCtrl) -> Result<(Handle, BotId)> {
    game.set_help(Some(&*HELP_MSG)).await?;
    game.set_config(CONFIG.disabled()).await?;
    game.set_status(Some("building".into())).await?;

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

            map.set(TIMMY_POS, TileKind::FLOOR);
            map
        })
        .await?;

    world.set_spawn(SPAWN_POS, Dir::W).await?;

    let timmy = world
        .create_bot(CreateBotRequest::new(DUMMY).at(TIMMY_POS).facing(Dir::S))
        .await?;

    game.join(world.clone()).await?;

    utils::map::reveal(store, &world, create_map).await?;

    game.set_config(CONFIG).await?;
    game.set_status(None).await?;

    Ok((world, timmy))
}

async fn create_map(
    seed: [u8; 32],
    progress: mpsc::Sender<Map>,
) -> Result<Map> {
    debug!(?seed);

    let mut map = Map::new(SIZE);
    let mut rng = ChaCha8Rng::from_seed(seed);

    utils::map::draw_borders(&mut map, AREA, &progress).await;
    utils::map::draw_maze(&mut map, &mut rng, &progress, AREA, TIMMY_POS).await;

    create_map_entrance(&mut map, &progress).await;

    Ok(map)
}

async fn create_map_entrance(map: &mut Map, progress: &mpsc::Sender<Map>) {
    map.line(
        ivec2(AREA.x as i32 - 1, AREA.y as i32 - 2),
        ivec2(AREA.x as i32 - 1 + ENTRANCE_LEN as i32, AREA.y as i32 - 2),
        TileKind::FLOOR,
    );

    _ = progress.send(map.clone()).await;

    map.line(
        ivec2(AREA.x as i32 - 1, AREA.y as i32 - 3),
        ivec2(AREA.x as i32 - 1 + ENTRANCE_LEN as i32, AREA.y as i32 - 3),
        TileKind::WALL_H,
    );

    _ = progress.send(map.clone()).await;

    map.line(
        ivec2(AREA.x as i32 - 1, AREA.y as i32 - 1),
        ivec2(AREA.x as i32 - 1 + ENTRANCE_LEN as i32, AREA.y as i32 - 1),
        TileKind::WALL_H,
    );

    _ = progress.send(map.clone()).await;

    map.line(
        ivec2(AREA.x as i32 - 1 + ENTRANCE_LEN as i32, AREA.y as i32 - 3),
        ivec2(AREA.x as i32 - 1 + ENTRANCE_LEN as i32, AREA.y as i32 - 1),
        TileKind::WALL_V,
    );
}

async fn watch(world: &Handle, timmy: BotId) -> Result<()> {
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
    use kartoffels_utils::Asserter;
    use std::path::Path;

    #[tokio::test]
    async fn map() {
        let dir = Path::new("src")
            .join("views")
            .join("index")
            .join("challenges")
            .join("ctrls")
            .join("acyclic_maze")
            .join("tests")
            .join("map");

        let (tx, _) = mpsc::channel(1);
        let map = create_map(Default::default(), tx).await.unwrap();

        Asserter::new(dir).assert("expected.txt", map.to_string());
    }
}
