#![feature(extract_if)]
#![feature(inline_const_pat)]
#![feature(let_chains)]
#![allow(clippy::result_unit_err)]

mod bot;
mod bots;
mod config;
mod handle;
mod map;
mod mode;
mod policy;
mod snapshots;
mod stats;
mod store;
mod theme;
mod utils;

mod cfg {
    pub const SIM_HZ: u32 = 64_000;
    pub const SIM_TICKS: u32 = 1024;
    pub const MAX_REQUEST_BACKLOG: usize = 1024;
}

pub mod prelude {
    pub use crate::bot::BotId;
    pub use crate::config::Config;
    pub use crate::handle::{Handle, Request};
    pub use crate::map::{Map, Tile, TileBase};
    pub use crate::snapshots::{
        Snapshot, SnapshotAliveBot, SnapshotAliveBots, SnapshotBots,
        SnapshotQueuedBot, SnapshotQueuedBots,
    };
    pub use crate::theme::{ArenaThemeConfig, DungeonThemeConfig, ThemeConfig};
    pub use crate::utils::Dir;
}

pub(crate) use self::bot::*;
pub(crate) use self::bots::*;
pub(crate) use self::config::*;
pub(crate) use self::handle::*;
pub(crate) use self::map::*;
pub(crate) use self::mode::*;
pub(crate) use self::policy::*;
pub(crate) use self::snapshots::*;
pub(crate) use self::store::*;
pub(crate) use self::theme::*;
pub(crate) use self::utils::*;
use anyhow::Result;
use kartoffels_utils::Metronome;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use tokio::runtime::Handle as TokioHandle;
use tokio::sync::{broadcast, mpsc, oneshot};

pub fn create(_path: &Path, config: Config) -> Result<()> {
    let _mode = config.mode.create();
    let _theme = config.theme.create();

    todo!();
}

pub fn spawn(path: &Path) -> Result<Handle> {
    let path = path.to_owned();
    let world = SerializedWorld::load(&path)?;

    let (tx, rx) = mpsc::channel(cfg::MAX_REQUEST_BACKLOG);
    let handle = Handle::new(tx, Arc::new(world.name.to_string()));

    let rt = TokioHandle::current();

    thread::spawn(move || {
        let mut world = World {
            bots: world.bots.into_owned(),
            updates: broadcast::Sender::new(2),
            events: Default::default(),
            map: world.map.into_owned(),
            mode: world.mode.into_owned(),
            name: Arc::new(world.name.into_owned()),
            path: Some(path),
            paused: false,
            policy: world.policy.into_owned(),
            rng: SmallRng::from_entropy(),
            rx,
            systems: Default::default(),
            theme: world.theme.into_owned(),
        };

        let _rt = rt.enter();
        let mut metronome = Metronome::new(cfg::SIM_HZ, cfg::SIM_TICKS);

        loop {
            world.tick();

            metronome.tick();
            metronome.wait();
        }
    });

    Ok(handle)
}

struct World {
    bots: Bots,
    events: Events,
    map: Map,
    mode: Mode,
    name: Arc<String>,
    path: Option<PathBuf>,
    paused: bool,
    policy: Policy,
    rng: SmallRng,
    rx: RequestRx,
    systems: Container,
    theme: Theme,
    updates: broadcast::Sender<Arc<Snapshot>>,
}

impl World {
    fn tick(&mut self) {
        handle::process_requests::run(self);

        if !self.paused {
            bots::spawn::run(self);
            bots::tick::run(self);
            bots::kill::run(self);
        }

        snapshots::broadcast::run(self);
        store::save::run(self);
        stats::run(self);
    }
}

#[derive(Debug)]
struct Shutdown {
    tx: oneshot::Sender<()>,
}
