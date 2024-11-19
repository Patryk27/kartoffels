use crate::{DeadBot, Event, KillBot, QueuedBot, World};
use itertools::Either;
use tracing::trace;

pub fn run(world: &mut World, cmd: KillBot) {
    let KillBot {
        killed,
        reason,
        killer,
    } = cmd;

    match &killed {
        Either::Left(id) => {
            trace!(killed=?id, ?reason, ?killer, "killing bot");
        }
        Either::Right(bot) => {
            trace!(killed=?bot, ?reason, ?killer, "killing bot");
        }
    }

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

    world.events.add(Event::BotKilled { id: killed.id });
    world.mode.on_bot_killed(killed.id, killer);

    killed.log(reason);

    let decision = if !killed.oneshot
        && world.policy.auto_respawn
        && world.bots.queued.len() < world.policy.max_queued_bots
    {
        Decision::Queue
    } else {
        Decision::Discard
    };

    match decision {
        Decision::Queue => {
            killed.log("requeued");

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
        }

        Decision::Discard => {
            world.bots.dead.add(DeadBot {
                events: killed.events.snapshot(),
                id: killed.id,
                serial: killed.serial.snapshot(),
            });
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Decision {
    Queue,
    Discard,
}
