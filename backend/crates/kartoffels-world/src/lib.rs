#![feature(cmp_minmax)]
#![feature(extract_if)]
#![feature(inline_const_pat)]
#![feature(let_chains)]
#![feature(type_alias_impl_trait)]
#![allow(clippy::result_unit_err)]

mod bot;
mod bots;
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
    pub const SIM_HZ: u32 = 64_000;
    pub const SIM_TICKS: u32 = 1024;
    pub const MAX_REQUEST_BACKLOG: usize = 1024;
    pub const MAX_EVENT_BACKLOG: usize = 128;
}

pub mod prelude {
    pub use crate::bot::BotId;
    pub use crate::config::Config;
    pub use crate::events::Event;
    pub use crate::handle::{
        EventStream, EventStreamExt, Handle, Request, SnapshotStream,
        SnapshotStreamExt,
    };
    pub use crate::map::{Map, Tile, TileBase};
    pub use crate::mode::{DeathmatchMode, DeathmatchModeConfig, ModeConfig};
    pub use crate::policy::Policy;
    pub use crate::snapshots::{
        Snapshot, SnapshotAliveBot, SnapshotAliveBots, SnapshotBots,
        SnapshotQueuedBot, SnapshotQueuedBots,
    };
    pub use crate::theme::{
        ArenaThemeConfig, DungeonTheme, DungeonThemeConfig, ThemeConfig,
    };
    pub use crate::utils::Dir;
}

pub(crate) use self::bot::*;
pub(crate) use self::bots::*;
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
use anyhow::Result;
use glam::IVec2;
use kartoffels_utils::{Id, Metronome};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use tokio::runtime::Handle as TokioHandle;
use tokio::sync::{broadcast, mpsc, oneshot, watch};
use tracing::{debug, info, span, Level};

pub fn create(config: Config, path: Option<&Path>) -> Handle {
    let mut rng = SmallRng::from_entropy();

    let name = Arc::new(config.name);
    let mode = config.mode.create();
    let theme = config.theme.create();
    let policy = config.policy;

    let id = Id::new(&mut rng);
    let map = theme.create_map(&mut rng);
    let path = path.map(|path| path.to_owned());

    let (handle, rx) = handle(id, name.clone());

    World {
        bots: Default::default(),
        events: handle.inner.events.clone(),
        map,
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
    }
    .spawn(id);

    handle
}

pub fn resume(id: Id, path: &Path) -> Result<Handle> {
    let path = path.to_owned();

    let world = SerializedWorld::load(&path)?;
    let bots = world.bots.into_owned();
    let map = world.map.into_owned();
    let mode = world.mode.into_owned();
    let policy = world.policy.into_owned();
    let theme = world.theme.into_owned();
    let name = Arc::new(world.name.into_owned());

    let (handle, rx) = handle(id, name.clone());

    World {
        bots,
        events: handle.inner.events.clone(),
        map,
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
    };

    (handle, rx)
}

struct World {
    bots: Bots,
    events: broadcast::Sender<Arc<Event>>,
    map: Map,
    mode: Mode,
    name: Arc<String>,
    path: Option<PathBuf>,
    paused: bool,
    policy: Policy,
    rng: SmallRng,
    rx: RequestRx,
    snapshots: watch::Sender<Arc<Snapshot>>,
    spawn: (Option<IVec2>, Option<Dir>),
    theme: Theme,
}

impl World {
    fn spawn(mut self, id: Id) {
        let rt = TokioHandle::current();
        let span = span!(Level::INFO, "world", %id);

        thread::spawn(move || {
            let _rt = rt.enter();
            let _span = span.enter();

            info!("ready");

            let mut metronome = Metronome::new(cfg::SIM_HZ, cfg::SIM_TICKS);
            let mut systems = Container::default();

            let shutdown = loop {
                match self.tick(&mut systems) {
                    ControlFlow::Continue(_) => {
                        metronome.tick();
                        metronome.wait();
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
            bots::spawn::run(self, systems.get_mut());
            bots::tick::run(self);
        }

        snapshots::broadcast::run(self, systems.get_mut());
        storage::save::run(self, systems.get_mut());
        stats::run(self, systems.get_mut());

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
