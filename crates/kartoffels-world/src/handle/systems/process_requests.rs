use crate::{
    AliveBot, BotId, Event, KillBot, QueuedBot, Request, Shutdown, World,
};
use anyhow::{anyhow, Result};
use glam::IVec2;
use kartoffels_vm as vm;
use std::borrow::Cow;
use std::ops::ControlFlow;
use std::sync::Arc;
use tokio::sync::mpsc::error::TryRecvError;

pub fn run(world: &mut World) -> ControlFlow<Shutdown, ()> {
    loop {
        match world.rx.try_recv() {
            Ok(Request::Pause { paused }) => {
                world.paused = paused;
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
    let fw = vm::Firmware::new(&src)?;
    let vm = vm::Runtime::new(fw);
    let mut bot = AliveBot::new(&mut world.rng, vm);

    bot.log("uploaded and queued");

    let id = loop {
        let id = BotId::new(&mut world.rng);

        if !world.bots.contains(id) {
            break id;
        }
    };

    if world.bots.queued.len() < world.policy.max_queued_bots {
        world.bots.queued.push(QueuedBot {
            id,
            pos,
            bot,
            requeued: false,
        });

        _ = world.events.send(Arc::new(Event::BotCreated { id }));

        Ok(id)
    } else {
        Err(anyhow!("too many robots queued, try again in a moment"))
    }
}
