use crate::{
    Clock, Map, Snapshot, SnapshotAliveBot, SnapshotAliveBots, SnapshotBots,
    SnapshotDeadBot, SnapshotDeadBots, SnapshotQueuedBot, SnapshotQueuedBots,
    Tile, TileKind, World,
};
use ahash::AHashMap;
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

pub fn run(world: &mut World, state: &mut State) {
    if Instant::now() < state.next_run_at {
        return;
    }

    let snapshot = {
        let bots = prepare_bots(world);
        let raw_map = world.map.clone();
        let map = prepare_map(&bots, world);
        let version = state.version;

        Arc::new(Snapshot {
            raw_map,
            map,
            bots,
            version,
        })
    };

    world.snapshots.send_replace(snapshot);

    state.next_run_at = match world.clock {
        Clock::Auto { .. } => Instant::now() + Duration::from_millis(50),
        Clock::Manual { .. } => Instant::now(),
    };

    state.version += 1;
}

fn prepare_bots(world: &mut World) -> SnapshotBots {
    SnapshotBots {
        alive: prepare_alive_bots(world),
        dead: prepare_dead_bots(world),
        queued: prepare_queued_bots(world),
    }
}

fn prepare_alive_bots(world: &mut World) -> SnapshotAliveBots {
    let scores = world.mode.scores();

    let entries: Vec<_> = world
        .bots
        .alive
        .iter_mut()
        .map(|bot| SnapshotAliveBot {
            id: bot.id,
            pos: bot.pos,
            dir: bot.dir,
            age: bot.timer.age(&world.clock),
            score: scores.get(&bot.id).copied().unwrap_or_default(),
            serial: bot.serial.buffer(),
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

    SnapshotAliveBots {
        entries,
        id_to_idx,
        idx_by_scores,
    }
}

fn prepare_dead_bots(world: &mut World) -> SnapshotDeadBots {
    let entries = world
        .bots
        .dead
        .iter_mut()
        .map(|entry| {
            let bot = SnapshotDeadBot {
                serial: entry.serial.clone(),
            };

            (entry.id, bot)
        })
        .collect();

    SnapshotDeadBots { entries }
}

fn prepare_queued_bots(world: &mut World) -> SnapshotQueuedBots {
    let entries = world
        .bots
        .queued
        .iter_mut()
        .map(|entry| {
            let bot = SnapshotQueuedBot {
                serial: entry.bot.serial.buffer(),
                place: entry.place + 1,
                requeued: entry.bot.requeued,
            };

            (entry.bot.id, bot)
        })
        .collect();

    SnapshotQueuedBots { entries }
}

fn prepare_map(bots: &SnapshotBots, world: &World) -> Map {
    let mut map = world.map.clone();

    for (idx, bot) in bots.alive().iter().enumerate() {
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

    for (pos, obj) in world.objects.iter() {
        map.set(pos, obj.kind);
    }

    map
}
