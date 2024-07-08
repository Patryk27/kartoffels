use crate::{
    BotEntry, Bots, BroadcastReceiverRx, Client, ClientBot, ClientBotUpdate,
    ClientConnectedBotUpdate, ClientUpdate, Map, World,
};
use std::collections::BTreeMap;
use std::mem;
use std::sync::Arc;
use web_time::{Duration, Instant};

pub struct State {
    next_tick_at: Instant,
}

impl Default for State {
    fn default() -> Self {
        Self {
            next_tick_at: Instant::now(),
        }
    }
}

pub fn run(world: &mut World, state: &mut State) {
    if Instant::now() < state.next_tick_at {
        return;
    }

    let update = prepare_update(world);

    world
        .clients
        .extract_if(|client| {
            handle_client(&world.map, &world.bots, update.clone(), client)
        })
        .for_each(drop);

    state.next_tick_at = Instant::now() + Duration::from_millis(50);
}

fn prepare_update(world: &mut World) -> ClientUpdate {
    let mode = Some(Arc::new(world.mode.state()));

    let map = if world.map.take_dirty() {
        Some(Arc::new(world.map.clone()))
    } else {
        None
    };

    let bots = {
        let bots: BTreeMap<_, _> = world
            .bots
            .alive
            .iter()
            .map(|entry| {
                let pos = entry.pos;
                let dir = entry.bot.motor.dir;
                let age = entry.bot.timer.age();

                (entry.id, ClientBotUpdate { pos, dir, age })
            })
            .collect();

        Some(Arc::new(bots))
    };

    ClientUpdate {
        mode,
        map,
        bots,
        bot: None,
    }
}

fn handle_client(
    map: &Map,
    bots: &Bots,
    mut update: ClientUpdate,
    client: &mut Client,
) -> bool {
    update.map = if mem::take(&mut client.is_fresh) && update.map.is_none() {
        Some(Arc::new(map.clone()))
    } else {
        update.map.clone()
    };

    update.bot = client
        .bot
        .as_mut()
        .and_then(|bot| handle_client_bot(bots, bot));

    client.tx.try_send(update).is_err()
}

fn handle_client_bot(
    bots: &Bots,
    bot: &mut ClientBot,
) -> Option<ClientConnectedBotUpdate> {
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

    match bots.get(bot.id)? {
        BotEntry::Queued(entry) => Some(ClientConnectedBotUpdate::Queued {
            place: entry.place + 1,
            requeued: entry.bot.requeued,
            events,
        }),

        BotEntry::Alive(entry) => Some(ClientConnectedBotUpdate::Alive {
            age: entry.bot.timer.age(),
            serial: entry.bot.serial.buffer.clone(),
            events,
        }),

        BotEntry::Dead => Some(ClientConnectedBotUpdate::Dead { events }),
    }
}
