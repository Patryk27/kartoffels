use crate::{
    BotEntry, Bots, BroadcastReceiverRx, Conn, ConnBot, ConnBotUpdate,
    ConnJoinedBotUpdate, ConnMsg, Map, Mode, World,
};
use std::collections::BTreeMap;
use std::mem;
use std::sync::Arc;
use web_time::{Duration, Instant};

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

    let msg = prepare_msg(&world.bots, &mut world.map, &world.mode);

    world
        .conns
        .extract_if(|conn| {
            process_conn(&world.bots, &world.map, msg.clone(), conn)
        })
        .for_each(drop);

    state.next_run_at = Instant::now() + Duration::from_millis(50);
}

fn prepare_msg(bots: &Bots, map: &mut Map, mode: &Mode) -> ConnMsg {
    let mode = Some(Arc::new(mode.state()));

    let map = if map.take_dirty() {
        Some(Arc::new(map.clone()))
    } else {
        None
    };

    let bots = {
        let bots: BTreeMap<_, _> = bots
            .alive
            .iter()
            .map(|entry| {
                let pos = entry.pos;
                let dir = entry.bot.motor.dir;
                let age = entry.bot.timer.age();

                (entry.id, ConnBotUpdate { pos, dir, age })
            })
            .collect();

        Some(Arc::new(bots))
    };

    ConnMsg {
        mode,
        map,
        bots,
        bot: None,
    }
}

fn process_conn(
    bots: &Bots,
    map: &Map,
    mut update: ConnMsg,
    conn: &mut Conn,
) -> bool {
    update.map = if mem::take(&mut conn.is_fresh) && update.map.is_none() {
        Some(Arc::new(map.clone()))
    } else {
        update.map.clone()
    };

    update.bot = conn.bot.as_mut().map(|bot| process_conn_bot(bots, bot));

    conn.tx.try_send(update).is_err()
}

fn process_conn_bot(bots: &Bots, bot: &mut ConnBot) -> ConnJoinedBotUpdate {
    let events = bot
        .events
        .as_mut()
        .map(|events| {
            events
                .init
                .drain(..)
                .chain(events.rx.recv_pending())
                .collect()
        })
        .unwrap_or_default();

    match bots.get(bot.id) {
        Some(BotEntry::Queued(entry)) => ConnJoinedBotUpdate::Queued {
            place: entry.place + 1,
            requeued: entry.bot.requeued,
            events,
        },

        Some(BotEntry::Alive(entry)) => ConnJoinedBotUpdate::Alive {
            age: entry.bot.timer.age(),
            serial: entry.bot.serial.buffer.clone(),
            events,
        },

        Some(BotEntry::Dead(_)) => ConnJoinedBotUpdate::Dead { events },
        None => ConnJoinedBotUpdate::Unknown,
    }
}
