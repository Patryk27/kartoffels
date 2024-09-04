use crate::{AliveBot, BotId, KillBot, QueuedBot, Request, Shutdown, World};
use anyhow::{anyhow, Result};
use glam::IVec2;
use kartoffels_vm as vm;
use std::ops::ControlFlow;
use tokio::sync::mpsc::error::TryRecvError;
use tracing::trace;

pub fn run(world: &mut World) -> ControlFlow<Shutdown, ()> {
    loop {
        match world.rx.try_recv() {
            Ok(Request::Pause { paused }) => {
                world.paused = paused;
            }

            Ok(Request::Shutdown { tx }) => {
                break ControlFlow::Break(Shutdown { tx: Some(tx) });
            }

            Ok(Request::CreateBot {
                src,
                pos,
                ephemeral,
                tx,
            }) => {
                _ = tx.send(create_bot(world, src, pos, ephemeral));
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

fn create_bot(
    world: &mut World,
    src: Vec<u8>,
    pos: Option<IVec2>,
    ephemeral: bool,
) -> Result<BotId> {
    let fw = vm::Firmware::new(&src)?;
    let vm = vm::Runtime::new(fw);
    let mut bot = AliveBot::new(&mut world.rng, vm, ephemeral);

    bot.log("uploaded and queued".into());

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

        trace!(?id, ?pos, "bot queued");

        Ok(id)
    } else {
        trace!(?id, ?pos, "bot discarded (queue full)");

        Err(anyhow!("too many robots queued, try again in a moment"))
    }
}
