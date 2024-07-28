use crate::{QueuedBot, World};
use glam::IVec2;
use tracing::debug;
use web_time::{Duration, Instant};

#[derive(Debug)]
struct State {
    next_tick_at: Instant,
}

impl Default for State {
    fn default() -> Self {
        Self {
            next_tick_at: Instant::now(),
        }
    }
}

pub fn run(world: &mut World) {
    let state = world.systems.get_mut::<State>();

    if Instant::now() < state.next_tick_at {
        return;
    }

    state.next_tick_at = Instant::now() + Duration::from_millis(16);

    if world.bots.queued.is_empty()
        || world.bots.alive.len() >= world.policy.max_alive_bots
    {
        return;
    }

    let Some(pos) = sample_pos(world) else {
        if world.spawn_point.is_some() {
            debug!("spawn point is taken - can't dequeue pending bot");
        } else {
            debug!("all tiles are taken - can't dequeue pending bot");
        }

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

fn sample_pos(world: &mut World) -> Option<IVec2> {
    let mut nth = 0;

    loop {
        let pos = world
            .spawn_point
            .unwrap_or_else(|| world.map.rand_pos(&mut world.rng));

        if world.map.get(pos).is_floor()
            && world.bots.alive.lookup_by_pos(pos).is_none()
        {
            return Some(pos);
        }

        nth += 1;

        if nth >= 1024 {
            return None;
        }
    }
}
