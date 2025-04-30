use crate::*;
use tokio::sync::mpsc::error::TryRecvError;

pub fn communicate(world: &mut World) {
    world.fuel.tick(&world.clock);

    loop {
        let request = match world.clock {
            Clock::Manual { .. } => {
                if !world.fuel.is_empty() {
                    return;
                }

                world
                    .requests
                    .blocking_recv()
                    .ok_or(TryRecvError::Disconnected)
            }

            _ => world.requests.try_recv(),
        };

        if let Ok(request) = &request {
            debug!(?request, "processing");
        }

        match request {
            Ok(Request::Tick { fuel, tx }) => {
                world.fuel.set(fuel, tx);
            }

            Ok(Request::Pause { tx }) => {
                world.paused = true;

                _ = tx.send(());
            }

            Ok(Request::Resume { tx }) => {
                world.paused = false;

                _ = tx.send(());
            }

            Ok(Request::Shutdown { tx }) => {
                world.shutdown = Some(Shutdown { tx: Some(tx) });

                return;
            }

            Ok(Request::Rename { name: new_name, tx }) => {
                world.name.store(Arc::new(new_name));

                _ = tx.send(());
            }

            Ok(Request::CreateBot { req, tx }) => {
                _ = tx.send(world.bots.create(
                    &world.clock,
                    &mut world.events,
                    &mut world.lives,
                    &world.map,
                    &world.objects,
                    &world.policy,
                    &mut world.rng,
                    &world.spawn,
                    req,
                ));
            }

            Ok(Request::KillBot { id, reason, tx }) => {
                if let Some(bot) = world.bots.alive.remove(id) {
                    world.kill_bot(bot, reason, None);
                }

                _ = tx.send(());
            }

            Ok(Request::DeleteBot { id, tx }) => {
                world.bots.remove(id);

                _ = tx.send(());
            }

            Ok(Request::SetMap { map: new, tx }) => {
                world.map = new;

                _ = tx.send(());
            }

            Ok(Request::SetSpawn { pos, dir, tx }) => {
                world.spawn.pos = pos;
                world.spawn.dir = dir;

                _ = tx.send(());
            }

            Ok(Request::CreateObject { obj, pos, tx }) => {
                _ = tx.send(world.objects.create(&mut world.rng, obj, pos));
            }

            Ok(Request::DeleteObject { id, tx }) => {
                _ = tx.send(world.objects.remove(id));
            }

            Ok(Request::Overclock { clock, tx }) => {
                world.clock = clock;

                _ = tx.send(());
            }

            Err(TryRecvError::Empty) => {
                break;
            }

            Err(TryRecvError::Disconnected) => {
                world.shutdown = Some(Shutdown { tx: None });
                return;
            }
        }

        if let Clock::Manual { .. } = world.clock {
            break;
        }
    }
}
