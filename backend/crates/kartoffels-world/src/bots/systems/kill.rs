use crate::{DeadBot, KillBot, QueuedBot, World};
use itertools::Either;
use std::sync::Arc;

pub fn run(world: &mut World, cmd: KillBot) {
    let KillBot {
        killed,
        reason,
        killer,
    } = cmd;

    let mut killed = match killed {
        Either::Left(id) => {
            let Some(bot) = world.bots.alive.remove(id) else {
                // Mildly sus, but not fatal - this can happen when user tries
                // to restart a queued bot etc.
                return;
            };

            bot
        }

        Either::Right(bot) => *bot,
    };

    world.mode.on_bot_killed(killed.id, killer);
    killed.log(reason);

    if !killed.oneshot
        && world.policy.auto_respawn
        && world.bots.queued.len() < world.policy.max_queued_bots
    {
        world.bots.queued.push(QueuedBot {
            dir: None,
            events: killed.events,
            fw: killed.fw,
            id: killed.id,
            oneshot: false,
            pos: None,
            requeued: true,
            serial: killed.serial,
        });
    } else {
        killed.log("discarded");

        world.bots.dead.add(DeadBot {
            events: Arc::new(killed.events.into_entries()),
            id: killed.id,
            serial: killed.serial.buffer(),
        });
    }
}
