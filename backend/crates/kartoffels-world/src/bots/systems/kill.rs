use crate::{DeadBot, KillBot, QueuedBot, World};

pub fn run(world: &mut World, KillBot { id, reason, killer }: KillBot) {
    let Some(mut bot) = world.bots.alive.remove(id) else {
        // Mildly sus, but not fatal - this can happen when user tries to
        // restart a queued bot etc.
        return;
    };

    world.mode.on_bot_killed(id, killer);
    bot.log(reason);

    if let Some(killer) = killer {
        if let Some(killer) = world.bots.alive.get_mut(killer) {
            killer.bot.log(format!("stabbed {id}"));
        }
    }

    if !bot.oneshot
        && world.policy.auto_respawn
        && world.bots.queued.len() < world.policy.max_queued_bots
    {
        world.bots.queued.push(QueuedBot {
            dir: None,
            events: bot.events,
            fw: bot.fw,
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
