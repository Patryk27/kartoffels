use crate::{DeadBot, Event, KillBot, QueuedBot, World};
use std::sync::Arc;
use tracing::trace;

pub fn run(world: &mut World, KillBot { id, reason, killer }: KillBot) {
    trace!(?id, ?reason, ?killer, "bot killed");

    world.mode.on_bot_killed(id, killer);

    let mut bot = world.bots.alive.remove(id);

    bot.log(reason);

    if let Some(killer) = killer {
        if let Some(killer) = world.bots.alive.get_mut(killer) {
            killer.bot.log(format!("stabbed {}", id));
        }
    }

    _ = world.events.send(Arc::new(Event::BotKilled { id }));

    match bot.reset(&mut world.rng) {
        Ok(mut bot) => {
            if world.bots.queued.len() < world.policy.max_queued_bots {
                trace!(?id, "bot requeued");

                world.bots.queued.push(QueuedBot {
                    id,
                    bot,
                    pos: None,
                    requeued: true,
                });
            } else {
                let msg = "discarded (queue is full)";

                bot.log(msg.into());
                trace!(?id, "bot {}", msg);

                world.bots.dead.add(id, DeadBot { events: bot.events });
            }
        }

        Err(mut bot) => {
            let msg = "discarded";

            bot.log(msg.into());
            trace!(?id, "bot {}", msg);

            world.bots.dead.add(id, DeadBot { events: bot.events });
        }
    }
}
