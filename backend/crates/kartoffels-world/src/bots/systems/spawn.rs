use crate::{AliveBot, Bots, Dir, Map, Objects, QueuedBot, World};
use anyhow::{anyhow, Context, Result};
use glam::IVec2;
use rand::{Rng, RngCore};
use tracing::trace;

pub fn run(world: &mut World) {
    if world.bots.alive.count() >= world.policy.max_alive_bots {
        return;
    }

    let Some(bot) = world.bots.queued.peek() else {
        return;
    };

    let Some((pos, dir)) = determine_spawn_point(
        &mut world.rng,
        &world.map,
        &world.bots,
        &world.objects,
        world.spawn,
        bot,
    ) else {
        return;
    };

    // Unwrap-safety: We've just made sure that the queue is not empty
    let bot = world.bots.queued.pop().unwrap();
    let bot = AliveBot::new(&mut world.rng, pos, dir, bot);

    trace!(id=?bot.id, ?pos, ?dir, "spawning bot");

    world.bots.alive.add(bot);
}

// TODO parts of logic are duplicated with `run()`
pub fn run_now(world: &mut World, bot: QueuedBot) -> Result<()> {
    if world.bots.alive.count() >= world.policy.max_alive_bots {
        return Err(anyhow!("too many alive bots"));
    }

    let (pos, dir) = determine_spawn_point(
        &mut world.rng,
        &world.map,
        &world.bots,
        &world.objects,
        world.spawn,
        &bot,
    )
    .context("couldn't determine spawn point")?;

    let bot = AliveBot::new(&mut world.rng, pos, dir, bot);

    trace!(id=?bot.id, ?pos, ?dir, "spawning bot");

    world.bots.alive.add(bot);

    Ok(())
}

fn determine_spawn_point(
    rng: &mut impl RngCore,
    map: &Map,
    bots: &Bots,
    objs: &Objects,
    spawn: (Option<IVec2>, Option<Dir>),
    bot: &QueuedBot,
) -> Option<(IVec2, Dir)> {
    if let Some(pos) = bot.pos {
        let dir = bot.dir.unwrap_or_else(|| rng.gen());

        return if is_pos_legal(map, bots, objs, pos) {
            Some((pos, dir))
        } else {
            None
        };
    }

    if let Some(pos) = spawn.0 {
        let dir = spawn.1.unwrap_or_else(|| rng.gen());

        return if is_pos_legal(map, bots, objs, pos) {
            Some((pos, dir))
        } else {
            None
        };
    }

    sample_map(rng, map, bots, objs, bot)
}

fn sample_map(
    rng: &mut impl RngCore,
    map: &Map,
    bots: &Bots,
    objs: &Objects,
    bot: &QueuedBot,
) -> Option<(IVec2, Dir)> {
    let mut nth = 0;

    loop {
        let pos = map.sample_pos(rng);

        if is_pos_legal(map, bots, objs, pos) {
            let dir = bot.dir.unwrap_or_else(|| rng.gen());

            return Some((pos, dir));
        }

        nth += 1;

        if nth >= 1024 {
            return None;
        }
    }
}

fn is_pos_legal(map: &Map, bots: &Bots, objs: &Objects, pos: IVec2) -> bool {
    map.get(pos).is_floor()
        && bots.alive.lookup_at(pos).is_none()
        && objs.lookup_at(pos).is_none()
}
