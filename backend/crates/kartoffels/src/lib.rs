#![feature(array_chunks)]
#![feature(extract_if)]
#![feature(inline_const_pat)]
#![feature(let_chains)]
#![feature(try_blocks)]
#![allow(clippy::result_unit_err)]

mod bot;
mod bots;
mod cfg;
mod config;
mod handle;
mod map;
mod mode;
mod policy;
mod serde;
mod store;
mod systems;
mod theme;
mod update;
mod utils;
mod world;

pub mod iface {
    pub use crate::bot::BotId;
    pub use crate::config::Config;
    pub use crate::handle::Handle;
    pub use crate::utils::*;
    pub use crate::world::{WorldId, WorldName};
}

pub(crate) use self::bot::*;
pub(crate) use self::bots::*;
pub(crate) use self::config::*;
pub(crate) use self::handle::*;
pub(crate) use self::map::*;
pub(crate) use self::mode::*;
pub(crate) use self::policy::*;
pub(crate) use self::store::*;
pub(crate) use self::systems::*;
pub(crate) use self::theme::*;
pub(crate) use self::update::*;
pub(crate) use self::utils::*;
pub(crate) use self::world::*;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use tokio::runtime;
use tokio::sync::mpsc;
use tracing::{info, span, Level};

pub fn create(
    rt: runtime::Handle,
    id: WorldId,
    config: Config,
    path: Option<PathBuf>,
) -> Result<Handle> {
    let mut rng = rand::thread_rng();
    let mode = config.mode.create();
    let theme = config.theme.create();
    let policy = config.policy;

    info!(
        ?id,
        name = ?config.name,
        mode = ?mode.ty(),
        theme = ?theme.ty(),
        "creating world",
    );

    let map = theme.create_map(&mut rng);

    let world = World {
        id,
        name: Arc::new(config.name),
        mode,
        theme,
        policy,
        map,
        bots: Default::default(),
        path,
    };

    Ok(spawn(rt, world))
}

pub fn resume(rt: runtime::Handle, id: WorldId, path: &Path) -> Result<Handle> {
    let this = SerializedWorld::load(path)?;

    info!(
        ?id,
        name = ?*this.name,
        mode = ?this.mode.ty(),
        theme = ?this.theme.ty(),
        "resuming world",
    );

    let world = World {
        id,
        name: Arc::new(this.name.into_owned()),
        mode: this.mode.into_owned(),
        theme: this.theme.into_owned(),
        policy: this.policy.into_owned(),
        map: this.map.into_owned(),
        bots: this.bots.into_owned(),
        path: Some(path.to_owned()),
    };

    Ok(spawn(rt, world))
}

fn spawn(rt: runtime::Handle, world: World) -> Handle {
    let name = world.name.clone();
    let mode = world.mode.ty();
    let theme = world.theme.ty();

    let (tx, rx) = mpsc::channel(16 * 1024);

    thread::spawn({
        let span = span!(Level::INFO, "", world = world.id.to_string());

        move || {
            let _rt = rt.enter();
            let _span = span.enter();

            world.main(rx)
        }
    });

    Handle {
        name,
        mode,
        theme,
        tx,
    }
}

#[derive(Debug)]
struct World {
    id: WorldId,
    name: Arc<WorldName>,
    mode: Mode,
    theme: Theme,
    policy: Policy,
    map: Map,
    bots: Bots,
    path: Option<PathBuf>,
}

impl World {
    fn main(mut self, rx: RequestRx) {
        info!("ready");

        let mut rng = rand::thread_rng();
        let mut mtr = Metronome::new(cfg::SIM_HZ, cfg::SIM_TICKS);
        let mut comm = Communicator::new(rx);
        let mut ctrl = Controller;
        let mut stats = Statistician::new();
        let mut persist = Persistencer::new();
        let mut bcaster = Broadcaster::new();
        let mut antireap = Antireaper::new();

        loop {
            mtr.iter(|mtr| {
                comm.tick(&mut self, &mut rng, &mut bcaster);
                ctrl.tick(&mut self, &mut rng);
                stats.tick(&self, mtr, &bcaster);
                persist.tick(&self);
                bcaster.tick(&mut self);
                antireap.tick(&mut self);
            });
        }
    }
}
