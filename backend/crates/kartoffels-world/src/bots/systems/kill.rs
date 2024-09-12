use crate::{DeadBot, Event, KillBot, QueuedBot, World};
use std::sync::Arc;

pub fn run(world: &mut World, KillBot { id, reason, killer }: KillBot) {
    world.mode.on_bot_killed(id, killer);

    let mut bot = world.bots.alive.remove(id);

    bot.log(reason);

    if let Some(killer) = killer {
        if let Some(killer) = world.bots.alive.get_mut(killer) {
            killer.bot.log(format!("stabbed {}", id));
        }
    }

    _ = world.events.send(Arc::new(Event::BotKilled { id }));

    if world.policy.auto_respawn
        && world.bots.queued.len() < world.policy.max_queued_bots
    {
        world.bots.queued.push(QueuedBot {
            id,
            pos: None,
            requeued: true,
            events: bot.events,
            serial: bot.serial,
            cpu: bot.cpu.reset(),
        });
    } else {
        bot.log("discarded");

        world.bots.dead.add(id, DeadBot { events: bot.events });
    }
}
