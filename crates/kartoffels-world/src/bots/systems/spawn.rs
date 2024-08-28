use crate::{Bots, Map, QueuedBot, World};
use glam::IVec2;
use rand::RngCore;
use std::time::{Duration, Instant};
use tracing::trace;

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

    let pos = if let Some(pos) = bot.pos {
        if is_pos_valid(&world.map, &world.bots, pos) {
            pos
        } else {
            trace!(
                ?pos,
                "can't dequeue pending bot: requested spawn point is taken",
            );

            return;
        }
    } else if let Some(pos) =
        sample_pos(&mut world.rng, &world.map, &world.bots)
    {
        pos
    } else {
        trace!("can't dequeue pending bot: couldn't find empty tile");

        return;
    };

    // Unwrap-safety: We've just made sure that the queue is not empty
    let QueuedBot {
        id,
        requeued,
        mut bot,
        ..
    } = world.bots.queued.pop().unwrap();

    trace!(?id, ?pos, "bot dequeued and spawned");

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

        if is_pos_valid(map, bots, pos) {
            return Some(pos);
        }

        nth += 1;

        if nth >= 1024 {
            return None;
        }
    }
}

fn is_pos_valid(map: &Map, bots: &Bots, pos: IVec2) -> bool {
    map.get(pos).is_floor() && bots.alive.lookup_by_pos(pos).is_none()
}
