use crate::{bots, Clock, KillBot, Request, Shutdown, World};
use anyhow::anyhow;
use std::ops::ControlFlow;
use tokio::sync::mpsc::error::TryRecvError;
use tracing::debug;

pub fn run(world: &mut World) -> ControlFlow<Shutdown, ()> {
    loop {
        let request = match world.clock {
            Clock::Auto { .. } => world.rx.try_recv(),

            Clock::Manual { .. } => {
                if world.tick.is_some() {
                    world.rx.try_recv()
                } else {
                    world.rx.blocking_recv().ok_or(TryRecvError::Disconnected)
                }
            }
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
                break ControlFlow::Break(Shutdown { tx: Some(tx) });
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
                        id,
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

            Ok(Request::PutObject { pos, obj, tx }) => {
                world.objects.put(pos, obj);

                _ = tx.send(());
            }

            Ok(Request::Overclock { speed, tx }) => {
                if let Some(metronome) = &mut world.metronome {
                    metronome.overclock(speed);

                    _ = tx.send(Ok(()));
                } else {
                    _ = tx.send(Err(anyhow!(
                        "world's clock configuration doesn't allow for \
                         overclocking",
                    )));
                }
            }

            Err(TryRecvError::Empty) => {
                break ControlFlow::Continue(());
            }

            Err(TryRecvError::Disconnected) => {
                break ControlFlow::Break(Shutdown { tx: None });
            }
        }
    }
}
