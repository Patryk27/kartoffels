#![feature(array_chunks)]
#![feature(const_option)]
#![feature(extract_if)]
#![feature(inline_const_pat)]
#![feature(let_chains)]
#![feature(try_blocks)]
#![allow(clippy::result_unit_err)]

mod bot;
mod bots;
mod clients;
mod config;
mod handle;
mod map;
mod mode;
mod policy;
mod serde;
mod stats;
mod store;
mod theme;
mod utils;
mod world;

pub mod prelude {
    pub use crate::bot::BotId;
    pub use crate::config::Config;
    pub use crate::handle::Handle;
    pub use crate::utils::*;
    pub use crate::world::{WorldId, WorldName};
}

mod cfg {
    pub const SIM_HZ: u32 = 64_000;
    pub const SIM_TICKS: u32 = 1024;
    pub const VERSION: u32 = 3;
    pub const MAX_REQUEST_BACKLOG: usize = 16 * 1024;
}

pub(crate) use self::bot::*;
pub(crate) use self::bots::*;
pub(crate) use self::clients::*;
pub(crate) use self::config::*;
pub(crate) use self::handle::*;
pub(crate) use self::map::*;
pub(crate) use self::mode::*;
pub(crate) use self::policy::*;
pub(crate) use self::store::*;
pub(crate) use self::theme::*;
pub(crate) use self::utils::*;
pub(crate) use self::world::*;
use anyhow::Result;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;

pub fn create(
    id: WorldId,
    config: Config,
    path: Option<PathBuf>,
) -> Result<Handle> {
    let mut rng = rand::thread_rng();
    let mode = config.mode.create();
    let theme = config.theme.create();

    info!(
        ?id,
        name = ?config.name,
        mode = ?mode.ty(),
        theme = ?theme.ty(),
        "creating world",
    );

    let map = theme.create_map(&mut rng);
    let (tx, rx) = mpsc::channel(cfg::MAX_REQUEST_BACKLOG);

    let world = World {
        bots: Default::default(),
        clients: Default::default(),
        events: Default::default(),
        map,
        metronome: Metronome::new(cfg::SIM_HZ, cfg::SIM_TICKS),
        mode,
        name: Arc::new(config.name),
        path,
        paused: false,
        policy: config.policy,
        rng: SmallRng::from_entropy(),
        rx,
        spawn_point: None,
        systems: Default::default(),
        theme,

        #[cfg(target_arch = "wasm32")]
        web_interval_handle: Default::default(),
    };

    let handle = Handle::new(&world, tx);

    world.spawn();

    Ok(handle)
}

pub fn resume(id: WorldId, path: &Path) -> Result<Handle> {
    let this = SerializedWorld::load(path)?;

    info!(
        ?id,
        name = ?*this.name,
        mode = ?this.mode.ty(),
        theme = ?this.theme.ty(),
        "resuming world",
    );

    let (tx, rx) = mpsc::channel(cfg::MAX_REQUEST_BACKLOG);

    let world = World {
        bots: this.bots.into_owned(),
        clients: Default::default(),
        events: Default::default(),
        map: this.map.into_owned(),
        metronome: Metronome::new(cfg::SIM_HZ, cfg::SIM_TICKS),
        mode: this.mode.into_owned(),
        name: Arc::new(this.name.into_owned()),
        path: Some(path.to_owned()),
        paused: false,
        policy: this.policy.into_owned(),
        rng: SmallRng::from_entropy(),
        rx,
        spawn_point: None,
        systems: Default::default(),
        theme: this.theme.into_owned(),

        #[cfg(target_arch = "wasm32")]
        web_interval_handle: Default::default(),
    };

    let handle = Handle::new(&world, tx);

    world.spawn();

    Ok(handle)
}
