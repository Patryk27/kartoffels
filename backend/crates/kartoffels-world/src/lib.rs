#![feature(cmp_minmax)]
#![feature(inline_const_pat)]
#![feature(let_chains)]
#![feature(type_alias_impl_trait)]
#![allow(clippy::result_unit_err)]

mod bot;
mod bots;
mod clock;
mod config;
mod handle;
mod map;
mod mode;
mod objects;
mod policy;
mod snapshots;
mod stats;
mod storage;
mod theme;
mod utils;

mod cfg {
    pub const MAX_REQUEST_BACKLOG: usize = 128;
}

pub mod prelude {
    pub use crate::bot::BotId;
    pub use crate::clock::{Clock, ClockSpeed};
    pub use crate::config::Config;
    pub use crate::handle::{
        CreateBotRequest, Handle, Request, SnapshotStream,
    };
    pub use crate::map::{Map, MapBuilder, Tile, TileKind};
    pub use crate::mode::{DeathmatchMode, Mode};
    pub use crate::objects::{Object, ObjectKind};
    pub use crate::policy::Policy;
    pub use crate::snapshots::{
        Snapshot, SnapshotAliveBot, SnapshotAliveBots, SnapshotBot,
        SnapshotBots, SnapshotDeadBot, SnapshotDeadBots, SnapshotQueuedBot,
        SnapshotQueuedBots,
    };
    pub use crate::theme::{ArenaTheme, DungeonTheme, Theme};
    pub use crate::utils::Dir;
}

pub(crate) use self::bot::*;
pub(crate) use self::bots::*;
pub(crate) use self::clock::*;
pub(crate) use self::config::*;
pub(crate) use self::handle::*;
pub(crate) use self::map::*;
pub(crate) use self::mode::*;
pub(crate) use self::objects::*;
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
use rand::{Rng, SeedableRng};
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use tokio::runtime::Handle as TokioHandle;
use tokio::sync::{mpsc, oneshot, watch};
use tracing::{debug, info, info_span};

pub fn create(config: Config) -> Handle {
    assert!(
        !(config.path.is_some() && config.rng.is_some()),
        "enabling path and rng at the same time is not supported, because rng \
         state is currently not persisted"
    );

    let mut rng = config
        .rng
        .map(SmallRng::from_seed)
        .unwrap_or_else(SmallRng::from_entropy);

    let clock = config.clock;
    let id = rng.gen();
    let mode = config.mode;
    let name = Arc::new(config.name);
    let path = config.path;
    let policy = config.policy;
    let theme = config.theme;

    let map = theme
        .as_ref()
        .map(|theme| theme.create_map(&mut rng).unwrap())
        .unwrap_or_default();

    let (handle, rx) = handle(id, name.clone());

    World {
        bots: Default::default(),
        clock,
        map,
        metronome: clock.metronome(),
        mode,
        name,
        objects: Default::default(),
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
    let clock = Clock::default();
    let map = world.map.into_owned();
    let metronome = if bench { None } else { clock.metronome() };
    let mode = world.mode.into_owned();
    let name = Arc::new(world.name.into_owned());
    let policy = world.policy.into_owned();
    let theme = world.theme.map(|theme| theme.into_owned());

    let (handle, rx) = handle(id, name.clone());

    World {
        bots,
        clock,
        map,
        metronome,
        mode,
        name,
        objects: Default::default(),
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
            snapshots: Default::default(),
        }),
        permit: None,
    };

    (handle, rx)
}

struct World {
    bots: Bots,
    objects: Objects,
    clock: Clock,
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
        // We store bot indices into map's tile metadata and since those are u8,
        // we can't have than 256 bots
        assert!(self.policy.max_alive_bots <= 256);
        assert!(self.policy.max_queued_bots <= 256);

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
