use crate::{AnyBotUpdate, BotId, BotUpdate, World, WorldUpdate};
use std::collections::BTreeMap;
use std::mem;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Broadcaster {
    conns: Vec<Connection>,
    next_tick_at: Instant,
}

impl Broadcaster {
    pub fn new() -> Self {
        Self {
            conns: Default::default(),
            next_tick_at: Instant::now(),
        }
    }

    pub fn add(&mut self, id: Option<BotId>) -> mpsc::Receiver<WorldUpdate> {
        let (tx, rx) = mpsc::channel(32);

        self.conns.push(Connection {
            id,
            tx,
            is_fresh: true,
        });

        rx
    }

    pub fn len(&self) -> usize {
        self.conns.len()
    }

    pub fn tick(&mut self, world: &mut World) {
        if Instant::now() < self.next_tick_at {
            return;
        }

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
                .map(|bot| (bot.id, AnyBotUpdate { pos: bot.pos }))
                .collect();

            Some(Arc::new(bots))
        };

        self.conns
            .extract_if(|client| {
                let map = if mem::take(&mut client.is_fresh) && map.is_none() {
                    Some(Arc::new(world.map.clone()))
                } else {
                    map.clone()
                };

                let bot = client
                    .id
                    .and_then(|id| world.bots.alive.try_get(id))
                    .map(|bot| BotUpdate {
                        serial: bot.bot.serial.to_string(),
                    });

                let update = WorldUpdate {
                    mode: mode.clone(),
                    map,
                    bots: bots.clone(),
                    bot,
                };

                client.tx.try_send(update).is_err()
            })
            .for_each(drop);

        self.next_tick_at = Instant::now() + Duration::from_millis(100);
    }
}

#[derive(Debug)]
struct Connection {
    id: Option<BotId>,
    tx: mpsc::Sender<WorldUpdate>,
    is_fresh: bool,
}
