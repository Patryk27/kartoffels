use super::{Challenge, CONFIG};
use crate::views::game::{GameCtrl, HelpMsg, HelpMsgEvent};
use crate::{utils, Msg, MsgButton, MsgLine};
use anyhow::Result;
use futures::future::BoxFuture;
use glam::{ivec2, uvec2, IVec2, UVec2};
use kartoffels_prefabs::DUMMY;
use kartoffels_store::{Store, World};
use kartoffels_world::prelude::{
    BotId, Config, CreateBotRequest, Dir, Handle, Map, MapBuilder, Policy,
    TileKind,
};
use rand::RngCore;
use ratatui::style::Stylize;
use std::sync::LazyLock;
use termwiz::input::KeyCode;
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
             took timmy-bot's wheels and ran away",
        ),
        MsgLine::new(""),
        MsgLine::new("*show mercy*").centered(),
        MsgLine::new(""),
        MsgLine::new(
            "traverse the maze, find timmy and kill it — you'll be starting in \
             the bottom-right corner",
        ),
        MsgLine::new(""),
        MsgLine::new("difficulty: easy"),
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
    buttons: vec![HelpMsgEvent::close()],
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

        game.sync(world.version()).await?;
        game.msg(&COMPLETED_MSG).await?;

        Ok(())
    })
}

async fn init(store: &Store, game: &GameCtrl) -> Result<(World, BotId)> {
    game.set_help(Some(&*HELP_MSG)).await?;
    game.set_config(CONFIG.disabled()).await?;
    game.set_label(Some("building".into())).await?;

    let world = store
        .create_private_world(Config {
            policy: Policy {
                auto_respawn: false,
                max_alive_bots: 2,
                max_queued_bots: 1,
            },
            ..store.world_config("challenge:acyclic-maze")
        })
        .await?;

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

    game.join(&world).await?;

    utils::map::build(store, game, &world, create_map).await?;

    game.sync(world.version()).await?;
    game.set_config(CONFIG).await?;
    game.set_label(None).await?;

    Ok((world, timmy))
}

async fn create_map(mut rng: impl RngCore, mut map: MapBuilder) -> Result<Map> {
    map.begin(SIZE);

    utils::map::draw_borders(&mut map, AREA).await;
    utils::map::draw_maze(&mut rng, &mut map, AREA, TIMMY_POS).await;
    draw_entrance(&mut map).await;

    Ok(map.commit())
}

async fn draw_entrance(map: &mut MapBuilder) {
    map.line(
        ivec2(AREA.x as i32 - 1, AREA.y as i32 - 2),
        ivec2(AREA.x as i32 - 1 + ENTRANCE_LEN as i32, AREA.y as i32 - 2),
        TileKind::FLOOR,
    )
    .await;

    map.line(
        ivec2(AREA.x as i32 - 1, AREA.y as i32 - 3),
        ivec2(AREA.x as i32 - 1 + ENTRANCE_LEN as i32, AREA.y as i32 - 3),
        TileKind::WALL_H,
    )
    .await;

    map.line(
        ivec2(AREA.x as i32 - 1, AREA.y as i32 - 1),
        ivec2(AREA.x as i32 - 1 + ENTRANCE_LEN as i32, AREA.y as i32 - 1),
        TileKind::WALL_H,
    )
    .await;

    map.line(
        ivec2(AREA.x as i32 - 1 + ENTRANCE_LEN as i32, AREA.y as i32 - 3),
        ivec2(AREA.x as i32 - 1 + ENTRANCE_LEN as i32, AREA.y as i32 - 1),
        TileKind::WALL_V,
    )
    .await;
}

async fn watch(world: &Handle, timmy: BotId) -> Result<()> {
    let mut events = world.events()?;

    loop {
        if events.next_died_bot().await? == timmy {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kartoffels_utils::Asserter;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
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

        let (map, _) = MapBuilder::new();
        let rng = ChaCha8Rng::from_seed(Default::default());
        let map = create_map(rng, map).await.unwrap();

        Asserter::new(dir).assert("expected.txt", map.to_string());
    }
}
