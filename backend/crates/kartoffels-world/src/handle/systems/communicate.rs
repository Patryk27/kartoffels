use crate::{
    Bots, Clock, CreateBot, HandleRx, KillBot, Map, Objects, Paused, Request,
    Shutdown, Spawn, TickFuel, WorldRng,
};
use bevy_ecs::system::{Commands, ResMut};
use itertools::Either;
use tokio::sync::mpsc::error::TryRecvError;
use tracing::debug;

#[allow(clippy::too_many_arguments)]
pub fn communicate(
    mut bots: ResMut<Bots>,
    mut clock: ResMut<Clock>,
    mut cmds: Commands,
    mut map: ResMut<Map>,
    mut objects: ResMut<Objects>,
    mut paused: ResMut<Paused>,
    mut rng: ResMut<WorldRng>,
    mut rx: ResMut<HandleRx>,
    mut spawn: ResMut<Spawn>,
    mut tick_fuel: ResMut<TickFuel>,
) {
    let rx = &mut rx.0;

    if tick_fuel.dec() {
        return;
    }

    loop {
        let request = match *clock {
            Clock::Manual => {
                rx.blocking_recv().ok_or(TryRecvError::Disconnected)
            }

            _ => rx.try_recv(),
        };

        if let Ok(request) = &request {
            debug!(?request, "processing");
        }

        match request {
            Ok(Request::Tick { fuel, tx }) => {
                tick_fuel.set(fuel, tx);
            }

            Ok(Request::Pause { tx }) => {
                paused.set(true);

                _ = tx.send(());
            }

            Ok(Request::Resume { tx }) => {
                paused.set(false);

                _ = tx.send(());
            }

            Ok(Request::Shutdown { tx }) => {
                cmds.insert_resource(Shutdown { tx: Some(tx) });
                return;
            }

            Ok(Request::CreateBot { req, tx }) => {
                cmds.send_event(CreateBot {
                    req: Some(req),
                    tx: Some(tx),
                });
            }

            Ok(Request::KillBot { id, reason, tx }) => {
                cmds.send_event(KillBot {
                    killed: Either::Left(id),
                    reason,
                    killer: None,
                });

                _ = tx.send(());
            }

            Ok(Request::DeleteBot { id, tx }) => {
                bots.remove(id);

                _ = tx.send(());
            }

            Ok(Request::SetMap { map: new_map, tx }) => {
                *map = new_map;

                _ = tx.send(());
            }

            Ok(Request::SetSpawn { pos, dir, tx }) => {
                spawn.pos = pos;
                spawn.dir = dir;

                _ = tx.send(());
            }

            Ok(Request::CreateObject { obj, pos, tx }) => {
                _ = tx.send(objects.create(&mut rng.0, obj, pos));
            }

            Ok(Request::DeleteObject { id, tx }) => {
                _ = tx.send(objects.remove(id));
            }

            Ok(Request::Overclock {
                clock: new_clock,
                tx,
            }) => {
                *clock = new_clock;

                _ = tx.send(());
            }

            Err(TryRecvError::Empty) => {
                break;
            }

            Err(TryRecvError::Disconnected) => {
                cmds.insert_resource(Shutdown { tx: None });
                return;
            }
        }

        if let Clock::Manual = *clock {
            break;
        }
    }
}
