use crate::{bots, Event, KillBot, Request, Shutdown, World};
use std::ops::ControlFlow;
use std::sync::Arc;
use tokio::sync::mpsc::error::TryRecvError;

pub fn run(world: &mut World) -> ControlFlow<Shutdown, ()> {
    loop {
        match world.rx.try_recv() {
            Ok(Request::Pause) => {
                world.paused = true;
            }

            Ok(Request::Resume) => {
                world.paused = false;
            }

            Ok(Request::Shutdown { tx }) => {
                break ControlFlow::Break(Shutdown { tx: Some(tx) });
            }

            Ok(Request::CreateBot { src, pos, tx }) => {
                _ = tx.send(bots::create::run(world, src, pos));
            }

            Ok(Request::RestartBot { id }) => {
                bots::kill::run(
                    world,
                    KillBot {
                        id,
                        reason: "forcefully restarted".into(),
                        killer: None,
                    },
                );
            }

            Ok(Request::DestroyBot { id }) => {
                world.bots.remove(id);

                _ = world.events.send(Arc::new(Event::BotKilled { id }));
            }

            Ok(Request::SetSpawn { point, dir }) => {
                world.spawn = (point, dir);
            }

            Ok(Request::SetMap { map }) => {
                world.map = map;
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
