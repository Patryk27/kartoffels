use crate::{AliveBot, Bots, Dir, Event, Map, QueuedBot, World};
use glam::IVec2;
use rand::{Rng, RngCore};
use std::sync::Arc;

pub fn run(world: &mut World) {
    if world.bots.alive.len() >= world.policy.max_alive_bots {
        return;
    }

    let Some(bot) = world.bots.queued.peek() else {
        return;
    };

    let Some((pos, dir)) = determine_spawn_point(
        &mut world.rng,
        &world.map,
        &world.bots,
        world.spawn,
        bot,
    ) else {
        return;
    };

    // Unwrap-safety: We've just made sure that the queue is not empty
    let bot = world.bots.queued.pop().unwrap();
    let (bot, id) = AliveBot::spawn(&mut world.rng, bot, dir);

    world.bots.alive.add(id, pos, bot);

    _ = world.events.send(Arc::new(Event::BotSpawned { id }));
}

fn determine_spawn_point(
    rng: &mut impl RngCore,
    map: &Map,
    bots: &Bots,
    spawn: (Option<IVec2>, Option<Dir>),
    bot: &QueuedBot,
) -> Option<(IVec2, Dir)> {
    if let Some(pos) = bot.pos {
        return if is_pos_legal(map, bots, pos) {
            Some((pos, rng.gen()))
        } else {
            None
        };
    }

    if let Some(pos) = spawn.0 {
        let dir = spawn.1.unwrap_or_else(|| rng.gen());

        return if is_pos_legal(map, bots, pos) {
            Some((pos, dir))
        } else {
            None
        };
    }

    sample_map(rng, map, bots)
}

fn sample_map(
    rng: &mut impl RngCore,
    map: &Map,
    bots: &Bots,
) -> Option<(IVec2, Dir)> {
    let mut nth = 0;

    loop {
        let pos = map.sample_pos(rng);

        if is_pos_legal(map, bots, pos) {
            return Some((pos, rng.gen()));
        }

        nth += 1;

        if nth >= 1024 {
            return None;
        }
    }
}

fn is_pos_legal(map: &Map, bots: &Bots, pos: IVec2) -> bool {
    map.get(pos).is_floor() && bots.alive.get_by_pos(pos).is_none()
}
