#![feature(cmp_minmax)]
#![feature(debug_closure_helpers)]
#![feature(duration_constructors)]
#![feature(if_let_guard)]
#![feature(let_chains)]
#![feature(type_alias_impl_trait)]
#![allow(clippy::result_unit_err)]
#![allow(clippy::too_many_arguments)]

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
mod stats;
mod store;
mod theme;
mod utils;

pub mod prelude {
    pub use crate::bot::{BotEvent, BotId};
    pub use crate::clock::Clock;
    pub use crate::config::{
        Config, EVENT_STREAM_CAPACITY, MAX_LIVES_PER_BOT,
        REQUEST_STREAM_CAPACITY,
    };
    pub use crate::events::{Event, EventEnvelope, EventStream};
    pub use crate::handle::{CreateBotRequest, Handle, Request};
    pub use crate::map::{Map, MapBuilder, Tile, TileKind};
    pub use crate::object::{Object, ObjectId, ObjectKind};
    pub use crate::policy::Policy;
    pub use crate::snapshots::{
        AliveBotSnapshot, AliveBotsSnapshot, BotSnapshot, BotsSnapshot,
        DeadBotSnapshot, DeadBotsSnapshot, ObjectsSnapshot, QueuedBotSnapshot,
        QueuedBotsSnapshot, Snapshot, SnapshotStream,
    };
    pub use crate::store::WorldBuffer;
    pub use crate::theme::{ArenaTheme, CaveTheme, Theme};
    pub use crate::utils::AbsDir;
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
pub(crate) use self::store::*;
pub(crate) use self::theme::*;
pub(crate) use self::utils::*;
use ahash::AHashMap;
use anyhow::Result;
use arc_swap::ArcSwap;
use futures_util::FutureExt;
use maybe_owned::MaybeOwned;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::any::{Any, TypeId};
use std::ops::ControlFlow;
use std::sync::Arc;
use std::thread;
use tokio::sync::{broadcast, mpsc, watch};
use tracing::{info, Span};

pub fn create(config: Config) -> Handle {
    let mut rng = config
        .seed
        .map(ChaCha8Rng::from_seed)
        .unwrap_or_else(ChaCha8Rng::from_entropy);

    let map = config
        .theme
        .as_ref()
        .map(|theme| {
            theme
                .build(&mut rng, MapBuilder::detached())
                .now_or_never()
                .unwrap()
                .unwrap()
        })
        .unwrap_or_default();

    let boot = Bootstrap {
        bots: Default::default(),
        clock: config.clock,
        lives: Default::default(),
        map,
        name: Arc::new(ArcSwap::from_pointee(config.name)),
        policy: config.policy,
        rng,
        theme: config.theme,
    };

    spawn(boot, config.events)
}

pub fn resume(buf: WorldBuffer) -> Result<Handle> {
    let world = store::load(buf)?;

    let boot = Bootstrap {
        bots: world.bots.into_owned(),
        clock: Default::default(),
        lives: world.lives.into_owned(),
        map: world.map.into_owned(),
        name: Arc::new(ArcSwap::from_pointee(world.name)),
        policy: world.policy.into_owned(),
        rng: ChaCha8Rng::from_entropy(),
        theme: world.theme.map(MaybeOwned::into_owned),
    };

    Ok(spawn(boot, false))
}

fn spawn(boot: Bootstrap, with_events: bool) -> Handle {
    let (tx, rx) = mpsc::channel(REQUEST_STREAM_CAPACITY);

    let events =
        with_events.then(|| broadcast::Sender::new(EVENT_STREAM_CAPACITY));

    let snapshots = watch::Sender::default();

    let mut world = {
        let events = Events {
            tx: events.clone(),
            pending: Default::default(),
        };

        let snapshots = Snapshots {
            tx: snapshots.clone(),
        };

        boot.build(events, rx, snapshots)
    };

    let handle = Handle::new(SharedHandle {
        tx,
        name: world.name.clone(),
        events,
        snapshots,
    });

    let span = Span::current();

    thread::spawn(move || {
        let _span = span.entered();

        info!("ready");

        loop {
            if world.tick().is_break() {
                break;
            }
        }

        info!("shut down");
    });

    handle
}

struct Bootstrap {
    bots: Bots,
    clock: Clock,
    lives: Lives,
    map: Map,
    name: Arc<ArcSwap<String>>,
    policy: Policy,
    rng: ChaCha8Rng,
    theme: Option<Theme>,
}

impl Bootstrap {
    fn build(
        self,
        events: Events,
        requests: mpsc::Receiver<Request>,
        snapshots: Snapshots,
    ) -> World {
        let metronome = self.clock.metronome();

        World {
            bots: self.bots,
            clock: self.clock,
            events,
            fuel: Default::default(),
            lives: self.lives,
            map: self.map,
            metronome,
            name: self.name,
            objects: Default::default(), // TODO persist
            paused: Default::default(),
            policy: self.policy,
            requests,
            rng: self.rng,
            shutdown: Default::default(),
            snapshots,
            spawn: Default::default(),
            states: Default::default(),
            stats: Default::default(),
            theme: self.theme,
        }
    }
}

#[cfg(test)]
impl Default for Bootstrap {
    fn default() -> Self {
        Self {
            bots: Default::default(),
            clock: Default::default(),
            lives: Default::default(),
            map: Default::default(),
            name: Default::default(),
            policy: Default::default(),
            rng: ChaCha8Rng::from_seed(Default::default()),
            theme: Default::default(),
        }
    }
}

#[derive(Default)]
struct States {
    map: AHashMap<TypeId, Box<dyn Any + Send>>,
}

impl States {
    fn get_mut<T>(&mut self) -> &mut T
    where
        T: Any + Send + Default,
    {
        self.map
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(T::default()))
            .downcast_mut()
            .unwrap()
    }
}

struct World {
    bots: Bots,
    clock: Clock,
    events: Events,
    fuel: Fuel,
    lives: Lives,
    map: Map,
    metronome: Metronome,
    name: Arc<ArcSwap<String>>,
    objects: Objects, // TODO persist
    paused: bool,
    policy: Policy,
    requests: mpsc::Receiver<Request>,
    rng: ChaCha8Rng,
    shutdown: Option<Shutdown>,
    snapshots: Snapshots,
    spawn: Spawn,
    states: States,
    stats: Stats,
    theme: Option<Theme>,
}

impl World {
    fn tick(&mut self) -> ControlFlow<(), ()> {
        handle::communicate(self);

        if !self.paused {
            bots::dequeue(self);
            bots::tick(self);
        }

        stats::update(self);
        snapshots::send(self);
        lifecycle::log(self);

        if let Some(shutdown) = self.shutdown.take() {
            if let Some(tx) = shutdown.tx {
                _ = tx.send(store::save(self));
            }

            ControlFlow::Break(())
        } else {
            self.metronome.sleep(&self.clock);

            ControlFlow::Continue(())
        }
    }

    fn kill_bot(
        &mut self,
        killed: Box<AliveBot>,
        reason: String,
        killer: Option<BotId>,
    ) {
        self.bots.kill(
            &self.clock,
            &mut self.events,
            &mut self.lives,
            &self.policy,
            killed,
            reason,
            killer,
        );
    }

    fn cooldown(&mut self, base: u32) -> u32 {
        #[cfg(test)]
        let off = 0;

        #[cfg(not(test))]
        let off = base / 10;

        let min = base - off;
        let max = base + off;

        self.rng.gen_range(min..=max)
    }
}

#[cfg(test)]
impl Default for World {
    fn default() -> Self {
        let events = Events {
            tx: None,
            pending: Default::default(),
        };

        let (_, requests) = mpsc::channel(1);

        let snapshots = Snapshots {
            tx: watch::channel(Default::default()).0,
        };

        Bootstrap::default().build(events, requests, snapshots)
    }
}
