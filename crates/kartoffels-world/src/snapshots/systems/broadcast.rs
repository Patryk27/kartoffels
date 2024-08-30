use crate::{
    AliveBot, Map, Snapshot, SnapshotAliveBot, SnapshotAliveBots, SnapshotBots,
    SnapshotQueuedBot, SnapshotQueuedBots, Tile, TileBase, World,
};
use std::cmp::Reverse;
use std::sync::Arc;
use std::time::{Duration, Instant};

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

    let snap = Arc::new(Snapshot {
        map: prepare_map(world),
        bots: prepare_bots(world),
    });

    _ = world.updates.send(snap);

    world.systems.get_mut::<State>().next_run_at =
        Instant::now() + Duration::from_millis(50);
}

fn prepare_map(world: &World) -> Map {
    let mut map = world.map.clone();

    for (idx, entry) in world.bots.alive.iter().enumerate() {
        let pos = entry.pos;
        let dir = entry.bot.motor.dir;

        map.set(
            pos,
            Tile {
                base: TileBase::BOT,
                meta: [idx as u8, 0, 0],
            },
        );

        map.set(
            pos + dir.as_vec(),
            Tile {
                base: TileBase::BOT_CHEVRON,
                meta: [idx as u8, u8::from(dir), 0],
            },
        );
    }

    map
}

fn prepare_bots(world: &World) -> SnapshotBots {
    SnapshotBots {
        alive: prepare_alive_bots(world),
        queued: prepare_queued_bots(world),
    }
}

fn prepare_alive_bots(world: &World) -> SnapshotAliveBots {
    let entries: Vec<_> = world
        .bots
        .alive
        .iter()
        .map(|entry| SnapshotAliveBot {
            id: entry.id,
            pos: entry.pos,
            serial: render_serial(entry.bot),
            events: render_events(entry.bot),
            age: entry.bot.timer.age(),
        })
        .collect();

    let idx_lookup = world
        .bots
        .alive
        .iter()
        .enumerate()
        .map(|(idx, bot)| (bot.id, idx as u8))
        .collect();

    let idx_by_scores = {
        let scores = world.mode.scores();

        let mut idx: Vec<_> = world
            .bots
            .alive
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let score = scores.get(&entry.id).copied().unwrap_or_default();

                (score, idx as u8)
            })
            .collect();

        idx.sort_unstable_by_key(|(score, idx)| {
            let bot = &entries[*idx as usize];

            (Reverse(*score), Reverse(bot.age))
        });

        idx
    };

    SnapshotAliveBots {
        entries,
        idx_lookup,
        idx_by_scores,
    }
}

fn prepare_queued_bots(world: &World) -> SnapshotQueuedBots {
    let entries = world
        .bots
        .queued
        .iter()
        .map(|entry| {
            let bot = SnapshotQueuedBot {
                place: entry.place + 1,
                requeued: entry.bot.requeued,
            };

            (entry.id, bot)
        })
        .collect();

    SnapshotQueuedBots { entries }
}

// TODO handle 0xffffff00 and 0xffffff01
fn render_serial(bot: &AliveBot) -> String {
    let mut out = String::with_capacity(256);
    let mut len = 0;

    for ch in bot.serial.buffer.iter().copied() {
        let Some(ch) = char::from_u32(ch) else {
            continue;
        };

        out.push(ch);
        len += 1;

        if len % 16 == 15 {
            out.push('\n');
        }
    }

    out
}

fn render_events(bot: &AliveBot) -> Vec<String> {
    bot.events
        .iter()
        .map(|event| format!("> {}:\n{}", event.at, event.msg))
        .collect()
}
