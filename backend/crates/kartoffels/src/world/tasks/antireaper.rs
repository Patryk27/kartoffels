use crate::{QueuedBot, World};
use std::time::{Duration, Instant};
use tracing::debug;

#[derive(Debug)]
pub struct Antireaper {
    next_tick_at: Instant,
}

impl Antireaper {
    pub fn new() -> Self {
        Self {
            next_tick_at: Instant::now(),
        }
    }

    pub fn tick(&mut self, world: &mut World) {
        if Instant::now() < self.next_tick_at {
            return;
        }

        self.next_tick_at = Instant::now() + Duration::from_millis(125);

        if world.bots.queued.is_empty() || world.bots.alive.len() >= 64 {
            return;
        }

        let Some(pos) = world.bots.random_unoccupied_pos(&world.map) else {
            debug!("all tiles are taken - can't dequeue pending bot");
            return;
        };

        // Unwrap-safety: We've just make sure that queue is not empty
        let QueuedBot { id, bot } = world.bots.queued.pop().unwrap();

        debug!(?id, ?pos, "bot dequeued and spawned");

        world.bots.alive.add(id, pos, bot);
        // TODO remove from dead bots
    }
}
