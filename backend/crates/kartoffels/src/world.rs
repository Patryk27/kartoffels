mod handle;
mod metronome;
mod serialize;
mod tasks;
mod update;

pub use self::handle::*;
use self::metronome::*;
use self::serialize::*;
use self::tasks::*;
pub use self::update::*;
use crate::{
    BotId, Bots, Map, Mode, Policy, Theme, WorldConfig, WorldId, WorldName,
};
use anyhow::Result;
use derivative::Derivative;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use tokio::sync::mpsc::Receiver;
use tokio::sync::{mpsc, oneshot};
use tracing::{info, span, Level};

#[derive(Debug)]
pub struct World {
    pub(crate) id: WorldId,
    pub(crate) name: Arc<WorldName>,
    pub(crate) mode: Mode,
    pub(crate) theme: Theme,
    pub(crate) policy: Policy,
    pub(crate) map: Map,
    pub(crate) bots: Bots,
    pub(crate) path: Option<PathBuf>,
}

impl World {
    pub const SIM_HZ: u32 = 64_000;
    pub const SIM_TICKS: u32 = 256;

    pub fn create(
        id: WorldId,
        config: WorldConfig,
        path: Option<PathBuf>,
    ) -> Result<WorldHandle> {
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

        let this = Self {
            id,
            name: Arc::new(config.name),
            mode,
            theme,
            policy,
            map,
            bots: Default::default(),
            path,
        };

        // Insta-save to make sure we've got appropriate file permissions
        // (if not, better to fail fast)
        Persistencer::save(&this)?;

        Ok(this.spawn())
    }

    pub fn resume(id: WorldId, path: &Path) -> Result<WorldHandle> {
        let this = SerializedWorld::load(path)?;

        info!(
            ?id,
            name = ?*this.name,
            mode = ?this.mode.ty(),
            theme = ?this.theme.ty(),
            "resuming world",
        );

        let this = Self {
            id,
            name: Arc::new(this.name.into_owned()),
            mode: this.mode.into_owned(),
            theme: this.theme.into_owned(),
            policy: this.policy.into_owned(),
            map: this.map.into_owned(),
            bots: this.bots.into_owned(),
            path: Some(path.to_owned()),
        };

        // Insta-save to make sure we've got appropriate file permissions
        // (if not, better to fail fast)
        Persistencer::save(&this)?;

        Ok(this.spawn())
    }

    fn spawn(self) -> WorldHandle {
        let name = self.name.clone();
        let mode = self.mode.ty();
        let theme = self.theme.ty();

        let (tx, rx) = mpsc::channel(16 * 1024);

        thread::spawn({
            let span = span!(
                Level::INFO,
                "kartoffels",
                world = format!("{}/{}", self.id, self.name)
            );

            move || {
                let _span = span.enter();

                self.main(rx)
            }
        });

        WorldHandle {
            name,
            mode,
            theme,
            tx,
        }
    }

    fn main(mut self, rx: Receiver<WorldRequest>) {
        info!("ready");

        let mut rng = rand::thread_rng();
        let mut mtr = Metronome::new(Self::SIM_HZ, Self::SIM_TICKS);
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

#[derive(Derivative)]
#[derivative(Debug)]
enum WorldRequest {
    CreateBot {
        #[derivative(Debug = "ignore")]
        src: Vec<u8>,

        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<Result<BotId>>,
    },

    Join {
        id: Option<BotId>,

        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<mpsc::Receiver<WorldUpdate>>,
    },
}
