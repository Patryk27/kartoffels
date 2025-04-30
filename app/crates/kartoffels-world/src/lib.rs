#![feature(cmp_minmax)]
#![feature(debug_closure_helpers)]
#![feature(duration_constructors)]
#![feature(if_let_guard)]
#![feature(inline_const_pat)]
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
    pub use crate::bot::{BotEvent, BotId};
    pub use crate::clock::Clock;
    pub use crate::config::Config;
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
    pub use crate::theme::{ArenaTheme, CaveTheme, Theme};
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

use ahash::{AHashMap, AHashSet};
use anyhow::{anyhow, Context, Error, Result};
use arc_swap::ArcSwap;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use derivative::Derivative;
use futures_util::FutureExt;
use glam::{ivec2, uvec2, IVec2, IVec3, UVec2};
use itertools::Itertools;
use kartoffel as api;
use kartoffels_cpu::{Cpu, Firmware, Mmio};
use kartoffels_utils::Id;
use maybe_owned::MaybeOwned;
use rand::distributions::Standard;
use rand::prelude::Distribution;
use rand::seq::SliceRandom;
use rand::{Rng, RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::any::{Any, TypeId};
use std::cmp::Reverse;
use std::collections::{hash_map, VecDeque};
use std::fmt::Write as _;
use std::fs::File;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{BufReader, Cursor, Read, Write};
use std::ops::{ControlFlow, RangeInclusive};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{cmp, fmt, fs, iter, mem, ops, thread};
use tokio::sync::{broadcast, mpsc, oneshot, watch};
use tokio_stream::StreamExt;
use tracing::{debug, info, info_span, trace, warn, Span};

pub fn create(config: Config) -> Handle {
    let mut rng = config
        .seed
        .map(ChaCha8Rng::from_seed)
        .unwrap_or_else(ChaCha8Rng::from_entropy);

    let id = config.id.unwrap_or_else(|| rng.gen());

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
        id,
        lives: Default::default(),
        map,
        name: Arc::new(ArcSwap::from_pointee(config.name)),
        path: config.path,
        policy: config.policy,
        rng,
        theme: config.theme,
    };

    spawn(boot, config.events)
}

pub fn resume(id: Id, path: &Path) -> Result<Handle> {
    let world = storage::load(path)?;

    let boot = Bootstrap {
        bots: world.bots.into_owned(),
        clock: Default::default(),
        id,
        lives: world.lives.into_owned(),
        map: world.map.into_owned(),
        name: Arc::new(ArcSwap::from_pointee(world.name.into_owned())),
        path: Some(path.to_owned()),
        policy: world.policy.into_owned(),
        rng: ChaCha8Rng::from_entropy(),
        theme: world.theme.map(MaybeOwned::into_owned),
    };

    Ok(spawn(boot, false))
}

fn spawn(boot: Bootstrap, with_events: bool) -> Handle {
    let (tx, rx) = mpsc::channel(cfg::REQUEST_STREAM_CAPACITY);

    let events =
        with_events.then(|| broadcast::Sender::new(cfg::EVENT_STREAM_CAPACITY));

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
        id: world.id,
        name: world.name.clone(),
        events,
        snapshots,
    });

    thread::spawn(move || {
        let span = info_span!("world", id = %world.id);
        let _span = span.entered();

        info!(name=?world.name.load(), "ready");

        loop {
            if world.tick().is_break() {
                break;
            }
        }

        info!(name=?world.name.load(), "shut down");
    });

    handle
}

struct Bootstrap {
    bots: Bots,
    clock: Clock,
    id: Id,
    lives: Lives,
    map: Map,
    name: Arc<ArcSwap<String>>,
    path: Option<PathBuf>,
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
            id: self.id,
            lives: self.lives,
            map: self.map,
            metronome,
            name: self.name,
            objects: Default::default(), // TODO persist
            path: self.path,
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
            id: Id::new(1),
            lives: Default::default(),
            map: Default::default(),
            name: Default::default(),
            path: Default::default(),
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
    id: Id,
    lives: Lives,
    map: Map,
    metronome: Metronome,
    name: Arc<ArcSwap<String>>,
    objects: Objects, // TODO persist
    path: Option<PathBuf>,
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
        storage::save(self);
        lifecycle::log(self);

        if let Some(shutdown) = self.shutdown.take() {
            if let Some(tx) = shutdown.tx {
                _ = tx.send(());
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
