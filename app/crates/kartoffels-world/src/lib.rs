#![feature(cmp_minmax)]
#![feature(debug_closure_helpers)]
#![feature(duration_constructors)]
#![feature(if_let_guard)]
#![feature(inline_const_pat)]
#![feature(let_chains)]
#![feature(type_alias_impl_trait)]
#![allow(clippy::result_unit_err)]

mod bot;
mod bots;
mod clock;
mod config;
mod events;
mod handle;
mod lifecycle;
mod lives;
mod map;
mod object;
mod objects;
mod policy;
mod snapshots;
mod spec;
mod stats;
mod storage;
mod theme;
mod utils;

pub mod cfg {
    pub const EVENT_STREAM_CAPACITY: usize = 128;
    pub const REQUEST_STREAM_CAPACITY: usize = 128;
    pub const MAX_LIVES_PER_BOT: usize = 128;
}

pub mod prelude {
    pub use crate::bot::BotId;
    pub use crate::clock::Clock;
    pub use crate::config::Config;
    pub use crate::events::{Event, EventLetter, EventStream};
    pub use crate::handle::{CreateBotRequest, Handle, Request};
    pub use crate::map::{Map, MapBuilder, Tile, TileKind};
    pub use crate::object::{Object, ObjectId, ObjectKind};
    pub use crate::policy::Policy;
    pub use crate::snapshots::{
        AliveBotSnapshot, AliveBotsSnapshot, BotSnapshot, BotsSnapshot,
        DeadBotSnapshot, DeadBotsSnapshot, ObjectsSnapshot, QueuedBotSnapshot,
        QueuedBotsSnapshot, Snapshot, SnapshotStream,
    };
    pub use crate::theme::{ArenaTheme, DungeonTheme, Theme};
    pub use crate::utils::Dir;
}

pub(crate) use self::bot::*;
pub(crate) use self::bots::*;
pub(crate) use self::clock::*;
pub(crate) use self::config::*;
pub(crate) use self::events::*;
pub(crate) use self::handle::*;
pub(crate) use self::lifecycle::*;
pub(crate) use self::lives::*;
pub(crate) use self::map::*;
pub(crate) use self::object::*;
pub(crate) use self::objects::*;
pub(crate) use self::policy::*;
pub(crate) use self::snapshots::*;
pub(crate) use self::stats::*;
pub(crate) use self::storage::*;
pub(crate) use self::theme::*;
pub(crate) use self::utils::*;
use anyhow::Result;
use bevy_ecs::event::EventRegistry;
use bevy_ecs::schedule::{ExecutorKind, IntoSystemConfigs, Schedule};
use bevy_ecs::system::Res;
use bevy_ecs::world::World;
use kartoffels_utils::Id;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::path::Path;
use std::sync::Arc;
use std::thread;
use tokio::runtime::Handle as TokioHandle;
use tokio::sync::{broadcast, mpsc, watch};
use tracing::{info, info_span};

pub fn create(config: Config) -> Handle {
    config.validate();

    let mut rng = config
        .seed
        .map(ChaCha8Rng::from_seed)
        .unwrap_or_else(ChaCha8Rng::from_entropy);

    let id = rng.gen();

    let map = config
        .theme
        .as_ref()
        .map(|theme| theme.create_map(&mut rng).unwrap())
        .unwrap_or_default();

    let res = Resources {
        bots: Default::default(),
        clock: config.clock,
        id: WorldId(id),
        lives: Default::default(),
        map,
        name: WorldName(config.name),
        path: config.path.map(WorldPath),
        policy: config.policy,
        rng: WorldRng(rng),
        theme: config.theme,
    };

    create_or_resume(res, config.emit_events)
}

pub fn resume(id: Id, path: &Path) -> Result<Handle> {
    let world = storage::load(path)?;

    let res = Resources {
        bots: world.bots.into_owned(),
        clock: Default::default(),
        id: WorldId(id),
        lives: world.lives.into_owned(),
        map: world.map.into_owned(),
        name: WorldName(world.name.into_owned()),
        path: Some(WorldPath(path.to_owned())),
        policy: world.policy.into_owned(),
        rng: WorldRng(ChaCha8Rng::from_entropy()),
        theme: world.theme.map(|theme| theme.into_owned()),
    };

    Ok(create_or_resume(res, false))
}

struct Resources {
    bots: Bots,
    clock: Clock,
    id: WorldId,
    lives: Lives,
    map: Map,
    name: WorldName,
    path: Option<WorldPath>,
    policy: Policy,
    rng: WorldRng,
    theme: Option<Theme>,
}

fn create_or_resume(res: Resources, emit_events: bool) -> Handle {
    let mut world = create_world(res);
    let handle = create_handle(&mut world, emit_events);

    spawn(world);

    handle
}

fn create_world(res: Resources) -> World {
    let mut world = World::new();

    world.insert_resource(res.bots);
    world.insert_resource(res.clock.metronome());
    world.insert_resource(res.clock);
    world.insert_resource(res.id);
    world.insert_resource(res.map);
    world.insert_resource(res.name);
    world.insert_resource(res.policy);
    world.insert_resource(res.rng);
    world.insert_resource(res.lives);

    if let Some(path) = res.path {
        world.insert_resource(path);
    }

    if let Some(theme) = res.theme {
        world.insert_resource(theme);
    }

    world.insert_resource(Fuel::default());
    world.insert_resource(Objects::default()); // TODO persist
    world.insert_resource(Paused::default());
    world.insert_resource(Spawn::default());
    world.insert_resource(Stats::default());

    // ---

    EventRegistry::register_event::<CreateBot>(&mut world);
    EventRegistry::register_event::<Event>(&mut world);
    EventRegistry::register_event::<KillBot>(&mut world);
    EventRegistry::register_event::<SpawnBot>(&mut world);

    // ---

    world
}

fn create_handle(world: &mut World, emit_events: bool) -> Handle {
    let (tx, rx) = mpsc::channel(cfg::REQUEST_STREAM_CAPACITY);

    let events =
        emit_events.then(|| broadcast::Sender::new(cfg::EVENT_STREAM_CAPACITY));

    let snapshots = watch::Sender::default();

    // ---

    world.insert_resource(HandleRx(rx));

    world.insert_resource(Snapshots {
        tx: snapshots.clone(),
    });

    if let Some(events) = &events {
        world.insert_resource(Events {
            tx: events.clone(),
            pending: Default::default(),
        });
    }

    // ---

    let id = world.resource::<WorldId>().0;
    let name = world.resource::<WorldName>().0.clone();

    Handle {
        shared: Arc::new(HandleShared {
            tx,
            id,
            name,
            events,
            snapshots,
        }),
        permit: None,
    }
}

fn spawn(world: World) {
    let rt = TokioHandle::current();
    let id = world.resource::<WorldId>().0;
    let span = info_span!("world", %id);

    thread::spawn(move || {
        let _rt = rt.enter();
        let _span = span.enter();
        let schedule = main_schedule();

        main(world, schedule);
    });
}

fn schedule<M>(systems: impl IntoSystemConfigs<M>) -> Schedule {
    let mut schedule = Schedule::default();

    schedule.set_executor_kind(ExecutorKind::SingleThreaded);
    schedule.add_systems(systems.chain());
    schedule
}

fn main_schedule() -> Schedule {
    fn active(paused: Res<Paused>) -> bool {
        !paused.get()
    }

    schedule((
        handle::communicate,
        bots::create,
        bots::schedule_spawn.run_if(active),
        bots::spawn,
        bots::tick.run_if(active),
        bots::kill,
        lives::update,
        stats::update,
        events::track,
        snapshots::send,
        storage::save,
        lifecycle::log,
        clock::sleep,
        bevy_ecs::event::event_update_system,
    ))
}

fn main(mut world: World, mut schedule: Schedule) {
    info!(name=?world.resource::<WorldName>().0, "ready");

    let shutdown = loop {
        schedule.run(&mut world);

        if let Some(shutdown) = world.remove_resource::<Shutdown>() {
            break shutdown;
        }
    };

    if let Some(tx) = shutdown.tx {
        _ = tx.send(());
    }

    info!(name=?world.resource::<WorldName>().0, "shut down");
}
