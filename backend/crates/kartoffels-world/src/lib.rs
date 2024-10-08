#![feature(cmp_minmax)]
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
mod map;
mod mode;
mod policy;
mod snapshots;
mod stats;
mod storage;
mod theme;
mod utils;

mod cfg {
    pub const MAX_REQUEST_BACKLOG: usize = 128;
    pub const MAX_EVENT_BACKLOG: usize = 128;
}

pub mod prelude {
    pub use crate::bot::BotId;
    pub use crate::clock::{Clock, ClockSpeed};
    pub use crate::config::Config;
    pub use crate::events::Event;
    pub use crate::handle::{
        EventStream, EventStreamExt, Handle, Request, SnapshotStream,
        SnapshotStreamExt,
    };
    pub use crate::map::{Map, Tile, TileBase};
    pub use crate::mode::{DeathmatchMode, Mode};
    pub use crate::policy::Policy;
    pub use crate::snapshots::{
        Snapshot, SnapshotAliveBot, SnapshotAliveBots, SnapshotBots,
        SnapshotQueuedBot, SnapshotQueuedBots,
    };
    pub use crate::theme::{ArenaTheme, DungeonTheme, Theme};
    pub use crate::utils::Dir;

    pub static BOT_DUMMY: &[u8] = include_bytes!(env!("KARTOFFELS_BOT_DUMMY"));

    pub static BOT_ROBERTO: &[u8] =
        include_bytes!(env!("KARTOFFELS_BOT_ROBERTO"));
}

pub(crate) use self::bot::*;
pub(crate) use self::bots::*;
pub(crate) use self::clock::*;
pub(crate) use self::config::*;
pub(crate) use self::events::*;
pub(crate) use self::handle::*;
pub(crate) use self::map::*;
pub(crate) use self::mode::*;
pub(crate) use self::policy::*;
pub(crate) use self::snapshots::*;
pub(crate) use self::storage::*;
pub(crate) use self::theme::*;
pub(crate) use self::utils::*;
use crate::Metronome;
use anyhow::Result;
use glam::IVec2;
use kartoffels_utils::Id;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use tokio::runtime::Handle as TokioHandle;
use tokio::sync::{broadcast, mpsc, oneshot, watch};
use tracing::{debug, info, info_span};

pub fn create(config: Config) -> Handle {
    assert!(
        !(config.path.is_some() && config.rng.is_some()),
        "setting path and rng at the same time is not supported, because rng \
         state is not currently saved into the file system"
    );

    let mut rng = config
        .rng
        .map(SmallRng::from_seed)
        .unwrap_or_else(SmallRng::from_entropy);

    let clock = config.clock;
    let mode = config.mode;
    let name = Arc::new(config.name);
    let path = config.path;
    let policy = config.policy;
    let theme = config.theme;

    let id = Id::new(&mut rng);

    let map = theme
        .as_ref()
        .map(|theme| theme.create_map(&mut rng))
        .unwrap_or_default();

    let (handle, rx) = handle(id, name.clone());

    World {
        bots: Default::default(),
        clock,
        events: handle.inner.events.clone(),
        map,
        metronome: clock.metronome(false),
        mode,
        name,
        path,
        paused: false,
        policy,
        rng,
        rx,
        snapshots: handle.inner.snapshots.clone(),
        spawn: (None, None),
        theme,
        tick: None,
    }
    .spawn(id);

    handle
}

pub fn resume(id: Id, path: &Path, bench: bool) -> Result<Handle> {
    let path = path.to_owned();
    let world = SerializedWorld::load(&path)?;

    let bots = world.bots.into_owned();
    let clock = world.clock.into_owned();
    let map = world.map.into_owned();
    let mode = world.mode.into_owned();
    let name = Arc::new(world.name.into_owned());
    let policy = world.policy.into_owned();
    let theme = world.theme.map(|theme| theme.into_owned());

    let (handle, rx) = handle(id, name.clone());

    World {
        bots,
        clock,
        events: handle.inner.events.clone(),
        map,
        metronome: clock.metronome(bench),
        mode,
        name,
        path: Some(path),
        paused: false,
        policy,
        rng: SmallRng::from_entropy(),
        rx,
        snapshots: handle.inner.snapshots.clone(),
        spawn: (None, None),
        theme,
        tick: None,
    }
    .spawn(id);

    Ok(handle)
}

fn handle(id: Id, name: Arc<String>) -> (Handle, mpsc::Receiver<Request>) {
    let (tx, rx) = mpsc::channel(cfg::MAX_REQUEST_BACKLOG);

    let handle = Handle {
        inner: Arc::new(HandleInner {
            id,
            tx,
            name,
            events: broadcast::Sender::new(cfg::MAX_EVENT_BACKLOG),
            snapshots: watch::Sender::new(Default::default()),
        }),
        permit: None,
    };

    (handle, rx)
}

struct World {
    bots: Bots,
    clock: Clock,
    events: broadcast::Sender<Arc<Event>>,
    map: Map,
    metronome: Option<Metronome>,
    mode: Mode,
    name: Arc<String>,
    path: Option<PathBuf>,
    paused: bool,
    policy: Policy,
    rng: SmallRng,
    rx: RequestRx,
    snapshots: watch::Sender<Arc<Snapshot>>,
    spawn: (Option<IVec2>, Option<Dir>),
    theme: Option<Theme>,
    tick: Option<oneshot::Sender<()>>,
}

impl World {
    fn spawn(mut self, id: Id) {
        // We store bot indices into map's tile metadata - those are u8s, so
        // we can't index more than 256 bots
        assert!(self.policy.max_alive_bots <= 256);

        let rt = TokioHandle::current();
        let span = info_span!("world", %id);

        thread::spawn(move || {
            let _rt = rt.enter();
            let _span = span.enter();
            let mut systems = Container::default();

            info!(name=?self.name, "ready");

            let shutdown = loop {
                match self.tick(&mut systems) {
                    ControlFlow::Continue(_) => {
                        if let Some(metronome) = &mut self.metronome {
                            metronome.tick();
                            metronome.wait();
                        }
                    }

                    ControlFlow::Break(shutdown) => {
                        break shutdown;
                    }
                }
            };

            self.shutdown(&mut systems, shutdown);
        });
    }

    fn tick(&mut self, systems: &mut Container) -> ControlFlow<Shutdown, ()> {
        handle::process_requests::run(self)?;

        if !self.paused {
            bots::spawn::run(self);
            bots::tick::run(self);
        }

        snapshots::broadcast::run(self, systems.get_mut());
        storage::save::run(self, systems.get_mut());
        stats::run(self, systems.get_mut());

        if let Some(tick) = self.tick.take() {
            _ = tick.send(());
        }

        ControlFlow::Continue(())
    }

    fn shutdown(mut self, systems: &mut Container, shutdown: Shutdown) {
        debug!("shutting down");

        storage::save::run_now(&mut self, systems.get_mut(), true);

        if let Some(tx) = shutdown.tx {
            _ = tx.send(());
        }

        info!("shut down");
    }
}

#[derive(Debug)]
struct Shutdown {
    tx: Option<oneshot::Sender<()>>,
}
