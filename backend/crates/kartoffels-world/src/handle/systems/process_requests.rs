use crate::{bots, Clock, KillBot, Request, Shutdown, World};
use itertools::Either;
use std::ops::ControlFlow;
use tokio::sync::mpsc::error::TryRecvError;
use tracing::debug;

pub fn run(world: &mut World) -> ControlFlow<Shutdown, ()> {
    loop {
        let request = match world.clock {
            Clock::Manual => {
                world.rx.blocking_recv().ok_or(TryRecvError::Disconnected)
            }

            _ => world.rx.try_recv(),
        };

        if let Ok(request) = &request {
            debug!(?request, "processing");
        }

        match request {
            Ok(Request::Tick { tx }) => {
                assert!(world.tick.is_none());

                world.tick = Some(tx);
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
                return ControlFlow::Break(Shutdown { tx: Some(tx) });
            }

            Ok(Request::CreateBot { req, tx }) => {
                _ = tx.send(bots::create::run(world, req));
            }

            Ok(Request::CreateBots { reqs, tx }) => {
                let mut ids = Vec::new();

                for req in reqs {
                    ids.push(bots::create::run(world, req));
                }

                _ = tx.send(ids);
            }

            Ok(Request::KillBot { id, reason, tx }) => {
                bots::kill::run(
                    world,
                    KillBot {
                        killed: Either::Left(id),
                        reason,
                        killer: None,
                    },
                );

                _ = tx.send(());
            }

            Ok(Request::DeleteBot { id, tx }) => {
                world.bots.remove(id);

                _ = tx.send(());
            }

            Ok(Request::SetMap { map, tx }) => {
                world.map = map;

                _ = tx.send(());
            }

            Ok(Request::SetSpawn { point, dir, tx }) => {
                world.spawn = (point, dir);

                _ = tx.send(());
            }

            Ok(Request::CreateObject { obj, pos, tx }) => {
                let id = world.objects.create(&mut world.rng, obj, pos);

                _ = tx.send(id);
            }

            Ok(Request::DeleteObject { id, tx }) => {
                let result = world.objects.remove(id);

                _ = tx.send(result);
            }

            Ok(Request::Overclock { clock, tx }) => {
                world.clock = clock;

                _ = tx.send(());
            }

            Err(TryRecvError::Empty) => {
                return ControlFlow::Continue(());
            }

            Err(TryRecvError::Disconnected) => {
                return ControlFlow::Break(Shutdown { tx: None });
            }
        }

        if let Clock::Manual = world.clock {
            return ControlFlow::Continue(());
        }
    }
}
