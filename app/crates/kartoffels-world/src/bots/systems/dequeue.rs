use crate::World;

pub fn dequeue(world: &mut World) {
    if world.bots.alive.count() >= world.policy.max_alive_bots as usize {
        return;
    }

    let Some(bot) = world.bots.queued.pop_front() else {
        return;
    };

    if let Err((_, bot)) = world.bots.spawn(
        &world.clock,
        &mut world.events,
        &mut world.lives,
        &world.map,
        &world.objects,
        &mut world.rng,
        &world.spawn,
        bot,
    ) {
        world.bots.queued.push_front(bot);
    }
}
