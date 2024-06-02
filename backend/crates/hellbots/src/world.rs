mod handle;

pub use self::handle::*;
use crate::{
    AliveBot, BotId, Bots, LoopTimer, Map, Mode, Theme, WorldConfig, WorldId,
    WorldName, WorldSnapshot,
};
use anyhow::{Context, Result};
use derivative::Derivative;
use hellbots_vm as vm;
use maybe_owned::MaybeOwned;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::{mpsc, oneshot, RwLock};
use tracing::{debug, info, span, warn, Level};

#[derive(Debug)]
pub struct World {
    id: WorldId,
    name: Arc<WorldName>,
    mode: Mode,
    theme: Theme,
    map: Map,
    bots: Bots,
    snapshot: Arc<RwLock<WorldSnapshot>>,
    file: Option<File>,
}

impl World {
    const SIM_HZ: u32 = 64_000; // TODO make configurable
    const SIM_TICKS: u32 = 256; // TODO make configurable

    pub fn create(
        id: WorldId,
        config: WorldConfig,
        file: Option<File>,
    ) -> Result<WorldHandle> {
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

        let map = theme.create_map(&mut rng).context("create_map() failed")?;

        let this = Self {
            id,
            name: Arc::new(config.name),
            mode,
            theme,
            map,
            bots: Default::default(),
            snapshot: Default::default(),
            file,
        };

        Ok(this.spawn())
    }

    pub fn resume(id: WorldId, mut file: File) -> Result<WorldHandle> {
        let this = SerializedWorld::read_from(&mut file)?;

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
            map: this.map.into_owned(),
            bots: this.bots.into_owned(),
            snapshot: Default::default(),
            file: Some(file),
        };

        Ok(this.spawn())
    }

    fn spawn(self) -> WorldHandle {
        let name = self.name.clone();
        let mode = self.mode.ty();
        let theme = self.theme.ty();
        let snapshot = self.snapshot.clone();

        let (tx, rx) = mpsc::unbounded_channel();

        thread::spawn({
            let span = span!(
                Level::INFO,
                "hellbots",
                world = format!("{}/{}", self.id, self.name)
            );

            move || {
                let _span = span.enter();

                self.main(rx).expect("world actor has crashed")
            }
        });

        WorldHandle {
            name,
            mode,
            theme,
            tx,
            snapshot,
        }
    }

    fn main(mut self, mut rx: UnboundedReceiver<WorldMsg>) -> Result<()> {
        info!("ready");

        let mut rng = rand::thread_rng();
        let mut timer = LoopTimer::new(Self::SIM_HZ, Self::SIM_TICKS);
        let mut stats = StatsCtrl::new();
        let mut persistence = PersistenceCtrl::new();

        loop {
            timer.iter(|timer| -> Result<()> {
                self.process_msg(&mut rng, &mut rx)
                    .context("process_msg() failed")?;

                for _ in 0..Self::SIM_TICKS {
                    self.tick(&mut rng).context("tick() failed")?;
                }

                self.mode
                    .on_after_tick(&mut rng, &mut self.theme, &mut self.map)
                    .context("on_after_tick() failed")?;

                self.snapshot
                    .blocking_write()
                    .update(&self.mode, &self.map, &self.bots)
                    .context("update() failed")?;

                stats.process(&self, timer);
                persistence.process(&mut self)?;

                Ok(())
            })?;
        }
    }

    fn process_msg(
        &mut self,
        rng: &mut impl RngCore,
        rx: &mut UnboundedReceiver<WorldMsg>,
    ) -> Result<()> {
        let Ok(msg) = rx.try_recv() else {
            return Ok(());
        };

        info!(?msg, "processing message");

        match msg {
            WorldMsg::CreateBot { src, tx } => {
                match self.create_or_update_bot(rng, src, None) {
                    Ok(id) => _ = tx.send(Ok(id)),
                    Err(err) => _ = tx.send(Err(err)),
                }
            }

            WorldMsg::UpdateBot { id, src, tx } => {
                match self.create_or_update_bot(rng, src, Some(id)) {
                    Ok(_) => _ = tx.send(Ok(())),
                    Err(err) => _ = tx.send(Err(err)),
                }
            }
        }

        Ok(())
    }

    fn create_or_update_bot(
        &mut self,
        rng: &mut impl RngCore,
        src: Vec<u8>,
        id: Option<BotId>,
    ) -> Result<BotId> {
        let fw = vm::Firmware::new(&src)?;
        let vm = vm::Runtime::new(fw);
        let bot = AliveBot::new(rng, vm);
        let id = self.bots.submit(rng, &self.map, bot, id)?;

        Ok(id)
    }

    fn tick(&mut self, rng: &mut impl RngCore) -> Result<()> {
        for id in self.bots.alive.pick_ids(rng) {
            let Some((pos, bot, bots)) = self.bots.alive.entry_mut(id) else {
                continue;
            };

            match bot.tick(&self.map, &bots, pos) {
                Ok(state) => {
                    state
                        .apply(
                            rng,
                            &mut self.mode,
                            &mut self.map,
                            &mut self.bots,
                            id,
                            pos,
                        )
                        .context("apply() failed")
                        .with_context(|| {
                            format!("couldn't process bot {}", id)
                        })?;
                }

                Err(err) => {
                    let err = format!("{:?}", err);

                    self.bots
                        .kill(rng, &mut self.mode, &mut self.map, id, err, None)
                        .context("kill() failed")
                        .with_context(|| {
                            format!("couldn't process bot {}", id)
                        })?;
                }
            }
        }

        Ok(())
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Derivative)]
#[derivative(Debug)]
enum WorldMsg {
    CreateBot {
        #[derivative(Debug = "ignore")]
        src: Vec<u8>,

        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<Result<BotId>>,
    },

    UpdateBot {
        id: BotId,

        #[derivative(Debug = "ignore")]
        src: Vec<u8>,

        #[derivative(Debug = "ignore")]
        tx: oneshot::Sender<Result<()>>,
    },
}

#[derive(Serialize, Deserialize)]
struct SerializedWorld<'a> {
    // TODO version
    name: MaybeOwned<'a, WorldName>,
    mode: MaybeOwned<'a, Mode>,
    theme: MaybeOwned<'a, Theme>,
    map: MaybeOwned<'a, Map>,
    bots: MaybeOwned<'a, Bots>,
}

impl SerializedWorld<'_> {
    fn save_to(self, file: &mut File) -> Result<()> {
        file.seek(SeekFrom::Start(0))?;

        let mut writer = BufWriter::new(&mut *file);

        ciborium::into_writer(&self, &mut writer)?;

        writer.flush()?;
        drop(writer);

        let len = file.stream_position()?;

        file.set_len(len)?;

        Ok(())
    }

    fn read_from(file: &mut File) -> Result<Self> {
        let mut reader = BufReader::new(&mut *file);
        let this = ciborium::from_reader(&mut reader)?;

        Ok(this)
    }
}

#[derive(Debug)]
struct StatsCtrl {
    ticks: u32,
    next_check_at: Instant,
}

impl StatsCtrl {
    fn new() -> Self {
        Self {
            ticks: 0,
            next_check_at: Instant::now() + Duration::from_secs(1),
        }
    }

    fn process(&mut self, world: &World, timer: &LoopTimer) {
        self.ticks += World::SIM_TICKS;

        if Instant::now() < self.next_check_at {
            return;
        }

        let msg = format!(
            "{} bot(s) | {} KHz vCPU | {} ms backlog",
            world.bots.alive.len(),
            self.ticks / 1_000,
            timer.backlog_ms(),
        );

        if timer.backlog_ms() >= 500 {
            warn!("simulation is falling behind:");
            warn!("{}", msg);
        } else {
            debug!("{}", msg);
        }

        self.ticks = 0;
        self.next_check_at = Instant::now() + Duration::from_secs(1);
    }
}

#[derive(Debug)]
struct PersistenceCtrl {
    next_save_at: Instant,
}

impl PersistenceCtrl {
    fn new() -> Self {
        Self {
            next_save_at: Instant::now(),
        }
    }

    fn process(&mut self, world: &mut World) -> Result<()> {
        let Some(file) = &mut world.file else {
            return Ok(());
        };

        if Instant::now() < self.next_save_at {
            return Ok(());
        }

        debug!("saving");

        let world = SerializedWorld {
            name: MaybeOwned::Borrowed(&world.name),
            mode: MaybeOwned::Borrowed(&world.mode),
            theme: MaybeOwned::Borrowed(&world.theme),
            map: MaybeOwned::Borrowed(&world.map),
            bots: MaybeOwned::Borrowed(&world.bots),
        };

        let tt = Instant::now();

        world.save_to(file).context("couldn't save the world")?;

        info!(tt = ?tt.elapsed(), "saved");

        self.next_save_at = Instant::now() + Duration::from_secs(5);

        Ok(())
    }
}
