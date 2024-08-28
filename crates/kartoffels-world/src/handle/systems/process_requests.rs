use crate::{AliveBot, BotId, KillBot, QueuedBot, Request, Shutdown, World};
use anyhow::{anyhow, Result};
use glam::IVec2;
use kartoffels_vm as vm;
use tracing::{info, trace};

pub fn run(world: &mut World) {
    while let Ok(msg) = world.rx.try_recv() {
        match msg {
            Request::Listen { tx } => {
                _ = tx.send(world.updates.subscribe());
            }

            Request::Pause { paused } => {
                world.paused = paused;
            }

            Request::Shutdown { tx } => {
                info!("initiating shutdown");

                world.events.send(Shutdown { tx });
            }

            Request::CreateBot {
                src,
                pos,
                ephemeral,
                tx,
            } => {
                _ = tx.send(create_bot(world, src, pos, ephemeral));
            }

            Request::RestartBot { id } => {
                world.events.send(KillBot {
                    id,
                    reason: "forcefully restarted".into(),
                    killer: None,
                });
            }

            Request::DestroyBot { id } => {
                world.bots.remove(id);
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
