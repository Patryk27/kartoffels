use super::{CONFIG, Challenge};
use crate::views::game::{GameCtrl, HelpMsg};
use crate::{Msg, MsgBtn, MsgLine, utils};
use anyhow::Result;
use futures::future::BoxFuture;
use glam::{IVec2, UVec2, ivec2, uvec2};
use kartoffels_prefabs::DUMMY;
use kartoffels_store::{Store, World};
use kartoffels_world::prelude as w;
use ratatui::style::Stylize;
use std::sync::LazyLock;
use termwiz::input::KeyCode;
use tracing::info;

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
        MsgLine::new("--- *show mercy* ---").centered(),
        MsgLine::new(""),
        MsgLine::new(
            "starting from the bottom-right corner, traverse the maze, find \
             timmy, and kill it; sweet release",
        ),
        MsgLine::new(""),
        MsgLine::new("sounds easy, does it not?"),
        MsgLine::new("xoxo").italic().right_aligned(),
        MsgLine::new("the architects").italic().right_aligned(),
    ]
});

static START_MSG: LazyLock<Msg<bool>> =
    LazyLock::new(|| Msg::start(CHALLENGE.name, &DOCS));

static HELP_MSG: LazyLock<HelpMsg> = LazyLock::new(|| Msg::help(DOCS.clone()));

static CONGRATS_MSG: LazyLock<Msg> = LazyLock::new(|| {
    Msg::new(CHALLENGE.name)
        .line("congrats!")
        .line("")
        .line("poor timmy is in a better place now, all thanks to you")
        .btn(MsgBtn::enter("exit", ()))
        .build()
});

const AREA: UVec2 = uvec2(37, 19);
const ENTRANCE_LEN: u32 = 5;
const SIZE: UVec2 = uvec2(AREA.x + ENTRANCE_LEN, AREA.y);

const TIMMY_POS: IVec2 = ivec2(1, 1);
const SPAWN_POS: IVec2 = ivec2(35 + (ENTRANCE_LEN as i32), 17);

fn run(store: &Store, game: GameCtrl) -> BoxFuture<Result<()>> {
    info!("run()");

    Box::pin(async move {
        let msg = game.msg_ex(&START_MSG).await?;

        if *msg.answer() {
            msg.close().await?;
            main(store, game).await
        } else {
            Ok(())
        }
    })
}

async fn main(store: &Store, game: GameCtrl) -> Result<()> {
    let (world, timmy) = init(store, &game).await?;

    watch(&world, timmy).await?;

    game.sync(world.version()).await?;
    game.msg_ex(&CONGRATS_MSG).await?;

    Ok(())
}

async fn init(store: &Store, game: &GameCtrl) -> Result<(World, w::BotId)> {
    game.set_help(Some(&*HELP_MSG)).await?;
    game.set_config(CONFIG.disabled()).await?;
    game.set_label(Some("building".into())).await?;

    let world = store
        .create_private_world(w::Config {
            policy: w::Policy {
                allow_breakpoints: true,
                auto_respawn: false,
                max_alive_bots: 2,
                max_queued_bots: 1,
            },
            ..store.world_config("challenge:acyclic-maze")
        })
        .await?;

    world
        .set_map({
            let mut map = w::Map::new(SIZE);

            map.set(TIMMY_POS, w::TileKind::FLOOR);
            map
        })
        .await?;

    world.set_spawn(SPAWN_POS, w::AbsDir::W).await?;

    let timmy = world
        .create_bot(
            w::CreateBotRequest::new(DUMMY)
                .at(TIMMY_POS)
                .facing(w::AbsDir::S),
        )
        .await?;

    game.visit(&world).await?;

    // ---

    utils::map::build(store, game, &world, async |mut rng, mut map| {
        map.begin(SIZE);

        utils::map::draw_borders(&mut map, AREA).await;
        utils::map::draw_maze(&mut rng, &mut map, AREA, TIMMY_POS).await;
        draw_entrance(&mut map).await;

        Ok(map.commit())
    })
    .await?;

    // ---

    game.sync(world.version()).await?;
    game.set_config(CONFIG).await?;
    game.set_label(None).await?;

    Ok((world, timmy))
}

async fn draw_entrance(map: &mut w::MapBuilder) {
    map.line(
        ivec2(AREA.x as i32 - 1, AREA.y as i32 - 2),
        ivec2(AREA.x as i32 - 1 + ENTRANCE_LEN as i32, AREA.y as i32 - 2),
        w::TileKind::FLOOR,
    )
    .await;

    map.line(
        ivec2(AREA.x as i32 - 1, AREA.y as i32 - 3),
        ivec2(AREA.x as i32 - 1 + ENTRANCE_LEN as i32, AREA.y as i32 - 3),
        w::TileKind::WALL_H,
    )
    .await;

    map.line(
        ivec2(AREA.x as i32 - 1, AREA.y as i32 - 1),
        ivec2(AREA.x as i32 - 1 + ENTRANCE_LEN as i32, AREA.y as i32 - 1),
        w::TileKind::WALL_H,
    )
    .await;

    map.line(
        ivec2(AREA.x as i32 - 1 + ENTRANCE_LEN as i32, AREA.y as i32 - 3),
        ivec2(AREA.x as i32 - 1 + ENTRANCE_LEN as i32, AREA.y as i32 - 1),
        w::TileKind::WALL_V,
    )
    .await;
}

async fn watch(world: &w::Handle, timmy: w::BotId) -> Result<()> {
    let mut events = world.events()?;

    loop {
        if events.next_died_bot().await? == timmy {
            return Ok(());
        }
    }
}
