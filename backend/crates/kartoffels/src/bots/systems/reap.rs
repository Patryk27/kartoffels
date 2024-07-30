use crate::{DeadBot, KillBot, QueuedBot, World};
use tracing::debug;

pub fn run(world: &mut World) {
    while let Some(event) = world.events.recv() {
        run_now(world, event);
    }
}

pub(super) fn run_now(
    world: &mut World,
    KillBot { id, reason, killer }: KillBot,
) {
    debug!(?id, ?reason, ?killer, "bot killed");

    world.mode.on_bot_killed(id, killer);

    let mut bot = world.bots.alive.remove(id);

    bot.log(reason);

    if let Some(killer) = killer {
        if let Some(killer) = world.bots.alive.get_mut(killer) {
            killer.bot.log(format!("stabbed {}", id));
        }
    }

    match bot.reset(&mut world.rng) {
        Ok(mut bot) => {
            if world.bots.queued.len() < world.policy.max_queued_bots {
                debug!(?id, "bot requeued");

                world.bots.queued.push(QueuedBot {
                    id,
                    bot,
                    requeued: true,
                });
            } else {
                let msg = "discarded (queue is full)";

                bot.log(msg.into());
                debug!(?id, "bot {}", msg);

                world.bots.dead.add(id, DeadBot { events: bot.events });
            }
        }

        Err(mut bot) => {
            let msg = "discarded (firmware crashed)";

            bot.log(msg.into());
            debug!(?id, "bot {}", msg);

            world.bots.dead.add(id, DeadBot { events: bot.events });
        }
    }
}
