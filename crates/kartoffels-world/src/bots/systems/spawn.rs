use crate::{Bots, Event, Map, QueuedBot, World};
use glam::IVec2;
use rand::{Rng, RngCore};
use std::sync::Arc;
use std::time::{Duration, Instant};

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

    let Some(pos) = determine_spawn_point(
        &mut world.rng,
        &world.map,
        &world.bots,
        world.spawn.0,
        bot,
    ) else {
        return;
    };

    // Unwrap-safety: We've just made sure that the queue is not empty
    let QueuedBot {
        id,
        requeued,
        mut bot,
        ..
    } = world.bots.queued.pop().unwrap();

    bot.log(if requeued { "respawned" } else { "spawned" });

    bot.motor.dir = world.spawn.1.unwrap_or_else(|| world.rng.gen());

    world.bots.alive.add(id, pos, bot);

    _ = world.events.send(Arc::new(Event::BotSpawned { id }));
}

fn determine_spawn_point(
    rng: &mut impl RngCore,
    map: &Map,
    bots: &Bots,
    spawn_point: Option<IVec2>,
    bot: &QueuedBot,
) -> Option<IVec2> {
    if let Some(pos) = bot.pos {
        return if is_pos_valid(map, bots, pos) {
            Some(pos)
        } else {
            None
        };
    }

    if let Some(pos) = spawn_point {
        return if is_pos_valid(map, bots, pos) {
            Some(pos)
        } else {
            None
        };
    }

    sample_pos(rng, map, bots)
}

fn sample_pos(rng: &mut impl RngCore, map: &Map, bots: &Bots) -> Option<IVec2> {
    let mut nth = 0;

    loop {
        let pos = map.sample_pos(rng);

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
