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

        self.next_tick_at = Instant::now() + Duration::from_millis(16);

        if world.bots.queued.is_empty()
            || world.bots.alive.len() >= world.policy.max_alive_bots
        {
            return;
        }

        let Some(pos) = world.bots.random_unoccupied_pos(&world.map) else {
            debug!("all tiles are taken - can't dequeue pending bot");
            return;
        };

        // Unwrap-safety: We've just made sure that the queue is not empty
        let QueuedBot {
            id,
            requeued,
            mut bot,
        } = world.bots.queued.pop().unwrap();

        debug!(?id, ?pos, "bot dequeued and spawned");

        bot.log(if requeued {
            "respawned".into()
        } else {
            "spawned".into()
        });

        world.bots.alive.add(id, pos, bot);
    }
}
