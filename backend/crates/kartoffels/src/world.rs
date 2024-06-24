mod handle;
mod serialized;
mod update;

pub use self::handle::*;
use self::serialized::*;
pub use self::update::*;
use crate::{
    AliveBot, AliveBotEntryMut, BotId, Bots, LoopTimer, Map, Mode, Theme,
    WorldConfig, WorldId, WorldName,
};
use anyhow::{Context, Result};
use derivative::Derivative;
use kartoffels_vm as vm;
use maybe_owned::MaybeOwned;
use rand::RngCore;
use std::collections::BTreeMap;
use std::fs::File;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{mem, thread};
use tokio::sync::mpsc::Receiver;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, info, span, warn, Level};

#[derive(Debug)]
pub struct World {
    id: WorldId,
    name: Arc<WorldName>,
    mode: Mode,
    theme: Theme,
    map: Map,
    bots: Bots,
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
            file,
        };

        Ok(this.spawn())
    }

    pub fn resume(id: WorldId, mut file: File) -> Result<WorldHandle> {
        let this = SerializedWorld::load(&mut file)?;

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
            file: Some(file),
        };

        Ok(this.spawn())
    }

    fn spawn(self) -> WorldHandle {
        let name = self.name.clone();
        let mode = self.mode.ty();
        let theme = self.theme.ty();

        let (tx, rx) = mpsc::channel(256);

        thread::spawn({
            let span = span!(
                Level::INFO,
                "kartoffels",
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
        }
    }

    fn main(mut self, mut rx: Receiver<WorldRequest>) -> Result<()> {
        info!("ready");

        let mut rng = rand::thread_rng();
        let mut timer = LoopTimer::new(Self::SIM_HZ, Self::SIM_TICKS);
        let mut stats = WorldStats::new();
        let mut state = WorldState::new();
        let mut clients = WorldClients::new();

        loop {
            timer.iter(|timer| -> Result<()> {
                self.process_msg(&mut rng, &mut clients, &mut rx)
                    .context("process_msg() failed")?;

                for _ in 0..Self::SIM_TICKS {
                    self.tick(&mut rng).context("tick() failed")?;
                }

                self.mode
                    .on_after_tick(&mut rng, &mut self.theme, &mut self.map)
                    .context("on_after_tick() failed")?;

                stats.process(&self, timer, &clients);
                state.process(&mut self)?;
                clients.process(&mut self)?;

                Ok(())
            })?;
        }
    }

    fn process_msg(
        &mut self,
        rng: &mut impl RngCore,
        clients: &mut WorldClients,
        rx: &mut Receiver<WorldRequest>,
    ) -> Result<()> {
        let Ok(msg) = rx.try_recv() else {
            return Ok(());
        };

        info!(?msg, "processing message");

        match msg {
            WorldRequest::CreateBot { src, tx } => {
                _ = tx.send(
                    try {
                        let fw = vm::Firmware::new(&src)?;
                        let vm = vm::Runtime::new(fw);
                        let bot = AliveBot::new(rng, vm);

                        self.bots.create(rng, &self.map, bot)?
                    },
                );
            }

            WorldRequest::Join { id, tx } => {
                _ = tx.send(clients.add(id));
            }
        }

        Ok(())
    }

    fn tick(&mut self, rng: &mut impl RngCore) -> Result<()> {
        for id in self.bots.alive.pick_ids(rng) {
            let Some((AliveBotEntryMut { pos, bot }, bots)) =
                self.bots.alive.get_mut(id)
            else {
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

#[derive(Debug)]
struct WorldStats {
    ticks: u32,
    next_check_at: Instant,
}

impl WorldStats {
    fn new() -> Self {
        Self {
            ticks: 0,
            next_check_at: Instant::now() + Duration::from_secs(1),
        }
    }

    fn process(
        &mut self,
        world: &World,
        timer: &LoopTimer,
        clients: &WorldClients,
    ) {
        self.ticks += World::SIM_TICKS;

        if Instant::now() < self.next_check_at {
            return;
        }

        let msg = format!(
            "{} bot(s) | {} KHz vCPU | {} ms backlog | {} client(s)",
            world.bots.alive.len(),
            self.ticks / 1_000,
            timer.backlog_ms(),
            clients.len(),
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
struct WorldState {
    next_save_at: Instant,
}

impl WorldState {
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

        world.store(file).context("couldn't save the world")?;

        info!(tt = ?tt.elapsed(), "saved");

        self.next_save_at = Instant::now() + Duration::from_secs(5);

        Ok(())
    }
}

#[derive(Debug)]
struct WorldClients {
    clients: Vec<WorldClient>,
    next_update_at: Instant,
}

impl WorldClients {
    fn new() -> Self {
        Self {
            clients: Default::default(),
            next_update_at: Instant::now() + Duration::from_millis(25),
        }
    }

    fn add(&mut self, id: Option<BotId>) -> mpsc::Receiver<WorldUpdate> {
        let (tx, rx) = mpsc::channel(32);

        self.clients.push(WorldClient {
            id,
            tx,
            is_fresh: true,
        });

        rx
    }

    fn len(&self) -> usize {
        self.clients.len()
    }

    fn process(&mut self, world: &mut World) -> Result<()> {
        if Instant::now() < self.next_update_at {
            return Ok(());
        }

        let mode = Some(Arc::new(world.mode.state()?));

        let map = if world.map.take_dirty() {
            Some(Arc::new(world.map.clone()))
        } else {
            None
        };

        let bots = {
            let bots: BTreeMap<_, _> = world
                .bots
                .alive
                .iter()
                .map(|bot| (bot.id, AnyBotUpdate { pos: bot.pos }))
                .collect();

            Some(Arc::new(bots))
        };

        self.clients
            .extract_if(|client| {
                let map = if mem::take(&mut client.is_fresh) && map.is_none() {
                    Some(Arc::new(world.map.clone()))
                } else {
                    map.clone()
                };

                let bot = client
                    .id
                    .and_then(|id| world.bots.alive.get(id))
                    .map(|bot| BotUpdate {
                        serial: bot.bot.serial.to_string(),
                    });

                let update = WorldUpdate {
                    mode: mode.clone(),
                    map,
                    bots: bots.clone(),
                    bot,
                };

                match client.tx.try_send(update) {
                    Ok(_) => {
                        // Client is still alive, keep it
                        false
                    }

                    Err(_) => {
                        // Client has been disconnected, delete it
                        debug!("disconnecting client");
                        true
                    }
                }
            })
            .for_each(drop);

        self.next_update_at = Instant::now() + Duration::from_millis(100);

        Ok(())
    }
}

#[derive(Debug)]
struct WorldClient {
    id: Option<BotId>,
    tx: mpsc::Sender<WorldUpdate>,
    is_fresh: bool,
}
