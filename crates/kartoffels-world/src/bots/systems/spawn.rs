use crate::{Bots, Event, Map, QueuedBot, World};
use glam::IVec2;
use rand::RngCore;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::trace;

#[derive(Debug)]
pub struct State {
    next_run_at: Instant,
}

impl Default for State {
    fn default() -> Self {
        Self {
            next_run_at: Instant::now(),
        }
    }
}

pub fn run(world: &mut World, state: &mut State) {
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

    let pos = match determine_spawn_point(
        &mut world.rng,
        &world.map,
        &world.bots,
        world.spawn_point,
        bot,
    ) {
        Ok(pos) => pos,

        Err(err) => {
            trace!("can't dequeue pending bot: {}", err);
            return;
        }
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

    _ = world.events.send(Arc::new(Event::BotSpawned { id }));
}

fn determine_spawn_point(
    rng: &mut impl RngCore,
    map: &Map,
    bots: &Bots,
    spawn_point: Option<IVec2>,
    bot: &QueuedBot,
) -> Result<IVec2, &'static str> {
    if let Some(pos) = bot.pos {
        return if is_pos_valid(map, bots, pos) {
            Ok(pos)
        } else {
            Err("bot's spawn point is taken")
        };
    }

    if let Some(pos) = spawn_point {
        return if is_pos_valid(map, bots, pos) {
            Ok(pos)
        } else {
            Err("world's spawn point is taken")
        };
    }

    sample_pos(rng, map, bots).ok_or("couldn't find empty tile")
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
