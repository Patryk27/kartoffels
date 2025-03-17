use crate::{
    AliveBot, AliveBots, Bots, Clock, Dir, Event, Map, Objects, Policy,
    QueuedBot, Spawn, SpawnBot, WorldRng,
};
use anyhow::anyhow;
use bevy_ecs::event::EventMutator;
use bevy_ecs::system::{Commands, Res, ResMut};
use glam::{IVec2, UVec2};
use rand::{Rng, RngCore};
use tracing::trace;

pub fn schedule_spawn(
    mut cmds: Commands,
    mut bots: ResMut<Bots>,
    policy: Res<Policy>,
) {
    if bots.alive.count() >= policy.max_alive_bots {
        return;
    }

    let Some(bot) = bots.queued.pop_front() else {
        return;
    };

    cmds.send_event(SpawnBot {
        bot: Some(bot),
        tx: None,
        requeue_if_cant_spawn: true,
    });
}

#[allow(clippy::too_many_arguments)]
pub fn spawn(
    mut cmds: Commands,
    mut bots: ResMut<Bots>,
    clock: Res<Clock>,
    map: Res<Map>,
    objects: Res<Objects>,
    mut rng: ResMut<WorldRng>,
    spawn: Res<Spawn>,
    mut events: EventMutator<SpawnBot>,
) {
    for event in events.read() {
        let bot = event
            .bot
            .take()
            .expect("bot is missing - maybe event has been already processed");

        let Some((pos, dir)) = determine_spawn_point(
            &mut rng.0,
            &map,
            &bots.alive,
            &objects,
            &spawn,
            &bot,
        ) else {
            if let Some(tx) = event.tx.take() {
                _ = tx.send(Err(anyhow!("couldn't determine spawn point")));
            }

            if event.requeue_if_cant_spawn {
                bots.queued.push_front(bot);
            }

            continue;
        };

        let bot = AliveBot::new(&mut rng.0, &clock, pos, dir, *bot);
        let id = bot.id;

        trace!(?id, ?pos, ?dir, "spawning bot");

        cmds.send_event(Event::BotBorn { id });
        bots.alive.add(bot);

        if let Some(tx) = event.tx.take() {
            _ = tx.send(Ok(id));
        }
    }
}

fn determine_spawn_point(
    rng: &mut impl RngCore,
    map: &Map,
    bots: &AliveBots,
    objs: &Objects,
    spawn: &Spawn,
    bot: &QueuedBot,
) -> Option<(IVec2, Dir)> {
    if let Some(pos) = bot.pos {
        let dir = bot.dir.unwrap_or_else(|| rng.gen());

        return if is_pos_legal(map, bots, objs, pos, false) {
            Some((pos, dir))
        } else {
            None
        };
    }

    if let Some(pos) = spawn.pos {
        let dir = spawn.dir.unwrap_or_else(|| rng.gen());

        return if is_pos_legal(map, bots, objs, pos, false) {
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
    bots: &AliveBots,
    objs: &Objects,
    bot: &QueuedBot,
) -> Option<(IVec2, Dir)> {
    if map.size() == UVec2::ZERO {
        return None;
    }

    let mut idx = 0;

    loop {
        let pos = map.sample_pos(rng);

        if is_pos_legal(map, bots, objs, pos, true) {
            let dir = bot.dir.unwrap_or_else(|| rng.gen());

            return Some((pos, dir));
        }

        idx += 1;

        if idx >= 1024 {
            return None;
        }
    }
}

fn is_pos_legal(
    map: &Map,
    bots: &AliveBots,
    objs: &Objects,
    pos: IVec2,
    check_neighborhood: bool,
) -> bool {
    if !map.get(pos).is_floor() || objs.lookup_at(pos).is_some() {
        return false;
    }
    if !check_neighborhood {
        return bots.lookup_at(pos).is_none();
    }
    for x in -1..=1 {
        for y in -1..=1 {
            if bots.lookup_at(pos + IVec2::new(x, y)).is_some() {
                return false;
            }
        }
    }
    true
}
