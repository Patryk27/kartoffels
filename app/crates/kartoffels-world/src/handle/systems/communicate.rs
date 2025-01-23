use crate::{
    Bots, Clock, CreateBot, Fuel, HandleRx, KillBot, Map, Objects, Paused,
    Request, Shutdown, Spawn, WorldRng,
};
use bevy_ecs::system::{Commands, ResMut};
use tokio::sync::mpsc::error::TryRecvError;
use tracing::debug;

#[allow(clippy::too_many_arguments)]
pub fn communicate(
    mut bots: ResMut<Bots>,
    mut clock: ResMut<Clock>,
    mut cmds: Commands,
    mut fuel: ResMut<Fuel>,
    mut map: ResMut<Map>,
    mut objects: ResMut<Objects>,
    mut paused: ResMut<Paused>,
    mut rng: ResMut<WorldRng>,
    mut rx: ResMut<HandleRx>,
    mut spawn: ResMut<Spawn>,
) {
    fuel.tick(*clock);

    loop {
        let request = match *clock {
            Clock::Manual => {
                if !fuel.is_empty() {
                    return;
                }

                rx.0.blocking_recv().ok_or(TryRecvError::Disconnected)
            }

            _ => rx.0.try_recv(),
        };

        if let Ok(request) = &request {
            debug!(?request, "processing");
        }

        match request {
            Ok(Request::Tick {
                fuel: fuel_to_add,
                tx,
            }) => {
                fuel.set(fuel_to_add, tx);
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
                if let Some(bot) = bots.alive.remove(id) {
                    cmds.send_event(KillBot {
                        killed: Some(bot),
                        reason,
                        killer: None,
                    });
                }

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
