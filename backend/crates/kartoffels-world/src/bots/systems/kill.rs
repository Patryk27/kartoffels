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

    if !bot.oneshot
        && world.policy.auto_respawn
        && world.bots.queued.len() < world.policy.max_queued_bots
    {
        world.bots.queued.push(QueuedBot {
            cpu: bot.cpu.reset(),
            dir: None,
            events: bot.events,
            id,
            oneshot: false,
            pos: None,
            requeued: true,
            serial: bot.serial,
        });
    } else {
        bot.log("discarded");

        world.bots.dead.add(id, DeadBot { events: bot.events });
    }
}
