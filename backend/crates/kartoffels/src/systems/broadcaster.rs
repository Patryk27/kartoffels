use crate::{
    BotEntry, BotEntryMut, BotEvent, BotEventRx, BotId, BotUpdate,
    BroadcastReceiverRx, ConnectedBotUpdate, Update, UpdateRx, UpdateTx, World,
};
use std::collections::BTreeMap;
use std::mem;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Broadcaster {
    connections: Vec<Connection>,
    next_tick_at: Instant,
}

impl Broadcaster {
    pub fn new() -> Self {
        Self {
            connections: Default::default(),
            next_tick_at: Instant::now(),
        }
    }

    pub fn add(
        &mut self,
        world: &mut World,
        bot_id: Option<BotId>,
    ) -> UpdateRx {
        let (tx, rx) = mpsc::channel(32);

        let bot = bot_id.map(|id| {
            let events = world
                .bots
                .get_mut(id)
                .map(|bot| match bot {
                    BotEntryMut::Queued(bot) => &mut bot.bot.events,
                    BotEntryMut::Alive(bot) => &mut bot.events,
                    BotEntryMut::Dead(bot) => &mut bot.events,
                })
                .map(|events| ConnectedBotEvents {
                    rx: events.subscribe(),
                    init: events.iter().cloned().collect(),
                });

            ConnectedBot { id, events }
        });

        self.connections.push(Connection {
            tx,
            bot,
            is_fresh: true,
        });

        rx
    }

    pub fn len(&self) -> usize {
        self.connections.len()
    }

    pub fn tick(&mut self, world: &mut World) {
        if Instant::now() < self.next_tick_at {
            return;
        }

        let update = Self::prepare_update(world);

        self.connections
            .extract_if(|connnection| connnection.tick(world, update.clone()))
            .for_each(drop);

        self.next_tick_at = Instant::now() + Duration::from_millis(50);
    }

    fn prepare_update(world: &mut World) -> Update {
        let mode = Some(Arc::new(world.mode.state()));

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
                .map(|entry| {
                    let pos = entry.pos;
                    let dir = entry.bot.motor.dir;
                    let age = entry.bot.timer.age();

                    (entry.id, BotUpdate { pos, dir, age })
                })
                .collect();

            Some(Arc::new(bots))
        };

        Update {
            mode,
            map,
            bots,
            bot: None,
        }
    }
}

#[derive(Debug)]
struct Connection {
    tx: UpdateTx,
    bot: Option<ConnectedBot>,
    is_fresh: bool,
}

impl Connection {
    fn tick(&mut self, world: &World, mut update: Update) -> bool {
        update.map = if mem::take(&mut self.is_fresh) && update.map.is_none() {
            Some(Arc::new(world.map.clone()))
        } else {
            update.map.clone()
        };

        update.bot = self.bot.as_mut().and_then(|bot| bot.tick(world));

        self.tx.try_send(update).is_err()
    }
}

#[derive(Debug)]
struct ConnectedBot {
    id: BotId,
    events: Option<ConnectedBotEvents>,
}

impl ConnectedBot {
    fn tick(&mut self, world: &World) -> Option<ConnectedBotUpdate> {
        let events = self
            .events
            .as_mut()
            .map(|events| events.tick())
            .unwrap_or_default();

        match world.bots.get(self.id)? {
            BotEntry::Queued(entry) => Some(ConnectedBotUpdate::Queued {
                place: entry.place + 1,
                requeued: entry.bot.requeued,
                events,
            }),

            BotEntry::Alive(entry) => Some(ConnectedBotUpdate::Alive {
                age: entry.bot.timer.age(),
                serial: entry.bot.serial.buffer.clone(),
                events,
            }),

            BotEntry::Dead => Some(ConnectedBotUpdate::Dead { events }),
        }
    }
}

#[derive(Debug)]
struct ConnectedBotEvents {
    rx: BotEventRx,
    init: Vec<Arc<BotEvent>>,
}

impl ConnectedBotEvents {
    fn tick(&mut self) -> Vec<Arc<BotEvent>> {
        self.init.drain(..).chain(self.rx.recv_pending()).collect()
    }
}
