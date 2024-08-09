use crate::{Bots, Map, QueuedBot, World};
use glam::IVec2;
use rand::RngCore;
use tracing::debug;
use web_time::{Duration, Instant};

#[derive(Debug)]
struct State {
    next_run_at: Instant,
}

impl Default for State {
    fn default() -> Self {
        Self {
            next_run_at: Instant::now(),
        }
    }
}

pub fn run(world: &mut World) {
    let state = world.systems.get_mut::<State>();

    if Instant::now() < state.next_run_at {
        return;
    }

    state.next_run_at = Instant::now() + Duration::from_millis(16);

    if world.bots.alive.len() >= world.policy.max_alive_bots {
        return;
    }

    let Some(bot) = world.bots.queued.peek() else {
        return;
    };

    // ---

    let pos = bot
        .pos
        .or_else(|| sample_pos(&mut world.rng, &world.map, &world.bots));

    let Some(pos) = pos else {
        if bot.pos.is_some() {
            debug!(
                pos = ?bot.pos,
                "can't dequeue pending bot: requested spawn point is taken",
            );
        } else {
            debug!("can't dequeue pending bot: all tiles are taken");
        }

        return;
    };

    // Unwrap-safety: We've just made sure that the queue is not empty
    let QueuedBot {
        id,
        requeued,
        mut bot,
        ..
    } = world.bots.queued.pop().unwrap();

    debug!(?id, ?pos, "bot dequeued and spawned");

    bot.log(if requeued {
        "respawned".into()
    } else {
        "spawned".into()
    });

    world.bots.alive.add(id, pos, bot);
}

fn sample_pos(rng: &mut impl RngCore, map: &Map, bots: &Bots) -> Option<IVec2> {
    let mut nth = 0;

    loop {
        let pos = map.rand_pos(rng);

        if map.get(pos).is_floor() && bots.alive.lookup_by_pos(pos).is_none() {
            return Some(pos);
        }

        nth += 1;

        if nth >= 1024 {
            return None;
        }
    }
}
