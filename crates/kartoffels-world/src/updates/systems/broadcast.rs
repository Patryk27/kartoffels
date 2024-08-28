use crate::{
    AliveBot, BotStatusUpdate, BotUpdate, Map, Tile, TileBase, Update, World,
};
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

    let msg = Arc::new(Update {
        map: prepare_map(world),
        bots: prepare_bots(world),
    });

    _ = world.updates.send(msg);

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

fn prepare_bots(world: &World) -> Vec<BotUpdate> {
    // TODO handle 0xffffff00 and 0xffffff01
    let prepare_serial = |bot: &AliveBot| -> String {
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
    };

    let prepare_events = |bot: &AliveBot| -> Vec<String> {
        bot.events
            .iter()
            .map(|event| format!("> {}:\n{}", event.at, event.msg))
            .collect()
    };

    let alive = world.bots.alive.iter().map(|entry| BotUpdate {
        id: entry.id,
        serial: prepare_serial(entry.bot),
        events: prepare_events(entry.bot),
        status: BotStatusUpdate::Alive {
            age: entry.bot.timer.age(),
        },
    });

    let queued = world.bots.queued.iter().map(|entry| BotUpdate {
        id: entry.id,
        serial: prepare_serial(&entry.bot.bot),
        events: prepare_events(&entry.bot.bot),
        status: BotStatusUpdate::Queued {
            place: entry.place + 1,
            requeued: entry.bot.requeued,
        },
    });

    alive.chain(queued).collect()
}
