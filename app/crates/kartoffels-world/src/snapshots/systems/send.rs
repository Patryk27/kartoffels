use crate::{
    AliveBotSnapshot, AliveBots, AliveBotsSnapshot, Bots, BotsSnapshot, Clock,
    DeadBotSnapshot, DeadBots, DeadBotsSnapshot, Events, Lives, LivesSnapshot,
    Map, ObjectSnapshot, Objects, ObjectsSnapshot, QueuedBotSnapshot,
    QueuedBots, QueuedBotsSnapshot, Snapshot, Snapshots, Stats, StatsSnapshot,
    Tile, TileKind,
};
use ahash::AHashMap;
use bevy_ecs::system::{Local, Res, ResMut};
use std::cmp::Reverse;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct State {
    next_run_at: Instant,
    version: u64,
}

impl Default for State {
    fn default() -> Self {
        Self {
            next_run_at: Instant::now(),
            version: 0,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn send(
    mut state: Local<State>,
    mut bots: ResMut<Bots>,
    mut events: Option<ResMut<Events>>,
    clock: Res<Clock>,
    lives: Res<Lives>,
    map: Res<Map>,
    objects: Res<Objects>,
    snapshots: Res<Snapshots>,
    stats: Res<Stats>,
) {
    if Instant::now() < state.next_run_at {
        return;
    }

    state.version += 1;

    let snapshot = {
        let bots = BotsSnapshot {
            alive: prepare_alive_bots(&mut bots.alive, &lives),
            dead: prepare_dead_bots(&mut bots.dead),
            queued: prepare_queued_bots(&mut bots.queued),
        };

        let stats = StatsSnapshot {
            entries: stats.entries.clone(),
        };

        let lives = LivesSnapshot {
            entries: lives.entries.clone(),
        };

        let map = prepare_map(&bots, &map, &objects);
        let objects = prepare_objects(&objects);
        let tiles = map.clone();

        Arc::new(Snapshot {
            bots,
            clock: *clock,
            lives,
            map,
            objects,
            stats,
            tiles,
            version: state.version,
        })
    };

    snapshots.tx.send_replace(snapshot);

    if let Some(events) = &mut events {
        events.send(state.version);
    }

    state.next_run_at = match *clock {
        Clock::Manual => Instant::now(),
        _ => Instant::now() + Duration::from_millis(33),
    };
}

fn prepare_alive_bots(
    bots: &mut AliveBots,
    lives: &Lives,
) -> AliveBotsSnapshot {
    let entries: Vec<_> = bots
        .iter_mut()
        .map(|bot| AliveBotSnapshot {
            age: bot.age(),
            dir: bot.dir,
            events: bot.events.snapshot(),
            id: bot.id,
            pos: bot.pos,
            score: lives.curr_score(bot.id),
            serial: bot.serial.snapshot(),
        })
        .collect();

    let id_to_idx: AHashMap<_, _> = entries
        .iter()
        .enumerate()
        .map(|(idx, bot)| (bot.id, idx as u8))
        .collect();

    let idx_by_scores = {
        let mut idx: Vec<_> = (0..(entries.len() as u8)).collect();

        idx.sort_unstable_by_key(|idx| {
            let bot = &entries[*idx as usize];

            (Reverse(bot.score), Reverse(bot.age), bot.id)
        });

        idx
    };

    AliveBotsSnapshot {
        entries,
        id_to_idx,
        idx_by_scores,
    }
}

fn prepare_dead_bots(bots: &mut DeadBots) -> DeadBotsSnapshot {
    let entries = bots
        .iter_mut()
        .map(|entry| {
            let bot = DeadBotSnapshot {
                events: entry.events.clone(),
                serial: entry.serial.clone(),
            };

            (entry.id, bot)
        })
        .collect();

    DeadBotsSnapshot { entries }
}

fn prepare_queued_bots(bots: &mut QueuedBots) -> QueuedBotsSnapshot {
    let entries = bots
        .iter_mut()
        .map(|entry| {
            let bot = QueuedBotSnapshot {
                events: entry.bot.events.snapshot(),
                place: entry.place + 1,
                reincarnated: entry.bot.requeued,
                serial: entry.bot.serial.snapshot(),
            };

            (entry.bot.id, bot)
        })
        .collect();

    QueuedBotsSnapshot { entries }
}

fn prepare_map(bots: &BotsSnapshot, map: &Map, objects: &Objects) -> Map {
    let mut map = map.clone();

    for (idx, bot) in bots.alive.iter().enumerate() {
        let tile = Tile {
            kind: TileKind::BOT,
            meta: [idx as u8, 0, 0],
        };

        let chevron_pos = bot.pos + bot.dir;

        let chevron_tile = Tile {
            kind: TileKind::BOT_CHEVRON,
            meta: [idx as u8, u8::from(bot.dir), 0],
        };

        map.set(bot.pos, tile);

        if !map.get(chevron_pos).is_bot() {
            map.set(chevron_pos, chevron_tile);
        }
    }

    for obj in objects.iter() {
        if let Some(pos) = obj.pos {
            map.set(pos, obj.obj.kind);
        }
    }

    map
}

fn prepare_objects(objects: &Objects) -> ObjectsSnapshot {
    let objects = objects
        .iter()
        .map(|obj| ObjectSnapshot {
            id: obj.id,
            pos: obj.pos,
            obj: obj.obj,
        })
        .collect();

    ObjectsSnapshot { objects }
}
