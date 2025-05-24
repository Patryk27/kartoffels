use crate::{store, Clock, Request, Shutdown, World};
use tokio::sync::mpsc::error::TryRecvError;
use tracing::{debug, warn};

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

        let (span, request) = match request {
            Ok(request) => request,

            Err(TryRecvError::Empty) => {
                break;
            }

            Err(TryRecvError::Disconnected) => {
                warn!("world abandoned");
                world.shutdown = Some(Shutdown { tx: None });
                return;
            }
        };

        let _span = span.entered();

        debug!(?request, "handling request");

        match request {
            Request::Ping { tx } => {
                _ = tx.send(());
            }

            Request::Tick { fuel, tx } => {
                world.fuel.set(fuel, tx);
            }

            Request::Pause { tx } => {
                world.paused = true;

                _ = tx.send(());
            }

            Request::Resume { tx } => {
                world.paused = false;

                _ = tx.send(());
            }

            Request::Shutdown { tx } => {
                world.shutdown = Some(Shutdown { tx: Some(tx) });
                return;
            }

            Request::Save { tx } => {
                _ = tx.send(store::save(world));
            }

            Request::GetPolicy { tx } => {
                _ = tx.send(world.policy.clone());
            }

            Request::SetPolicy { policy, tx } => {
                world.policy = policy;

                _ = tx.send(());
            }

            Request::CreateBot { req, tx } => {
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

            Request::KillBot { id, reason, tx } => {
                if let Some(bot) = world.bots.alive.remove(id) {
                    world.kill_bot(bot, reason, None);
                }

                _ = tx.send(());
            }

            Request::DeleteBot { id, tx } => {
                world.bots.remove(id);

                _ = tx.send(());
            }

            Request::SetMap { map: new, tx } => {
                world.map = new;

                _ = tx.send(());
            }

            Request::SetSpawn { pos, dir, tx } => {
                world.spawn.pos = pos;
                world.spawn.dir = dir;

                _ = tx.send(());
            }

            Request::CreateObject { obj, pos, tx } => {
                _ = tx.send(world.objects.create(&mut world.rng, obj, pos));
            }

            Request::DeleteObject { id, tx } => {
                _ = tx.send(world.objects.remove(id));
            }

            Request::Overclock { clock, tx } => {
                world.clock = clock;

                _ = tx.send(());
            }
        }

        if let Clock::Manual { .. } = world.clock {
            break;
        }
    }
}
