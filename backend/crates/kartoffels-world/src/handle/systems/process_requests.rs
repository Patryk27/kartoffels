use crate::{
    BotEvents, BotId, Event, KillBot, QueuedBot, Request, Shutdown, World,
};
use anyhow::{anyhow, Result};
use glam::IVec2;
use kartoffels_cpu::Cpu;
use std::borrow::Cow;
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
                _ = tx.send(create_bot(world, src, pos));
            }

            Ok(Request::RestartBot { id }) => {
                crate::bots::kill::run(
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

// TODO move to `bots` system
fn create_bot(
    world: &mut World,
    src: Cow<'static, [u8]>,
    pos: Option<IVec2>,
) -> Result<BotId> {
    let cpu = Cpu::new(&src)?;

    let id = loop {
        let id = BotId::new(&mut world.rng);

        if !world.bots.contains(id) {
            break id;
        }
    };

    if world.bots.queued.len() < world.policy.max_queued_bots {
        let events = {
            let mut events = BotEvents::default();

            events.add("uploaded and queued");
            events
        };

        world.bots.queued.push(QueuedBot {
            id,
            pos,
            cpu,
            events,
            serial: Default::default(),
            requeued: false,
        });

        _ = world.events.send(Arc::new(Event::BotCreated { id }));

        Ok(id)
    } else {
        Err(anyhow!("too many robots queued, try again in a moment"))
    }
}
