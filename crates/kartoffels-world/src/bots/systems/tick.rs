use crate::{cfg, AliveBotEntryMut, KillBot, World};

pub fn run(world: &mut World) {
    for _ in 0..cfg::SIM_TICKS {
        tick(world);
    }

    world
        .mode
        .on_after_tick(&mut world.rng, &mut world.theme, &mut world.map);
}

fn tick(world: &mut World) {
    for id in world.bots.alive.pick_ids(&mut world.rng) {
        let Some(AliveBotEntryMut { pos, bot, locator }) =
            world.bots.alive.get_mut(id)
        else {
            // Our bot got killed in the meantime, happens
            continue;
        };

        let kill = match bot.tick(&mut world.rng, &world.map, &locator, pos) {
            Ok(state) => state.apply(world, id, pos),

            Err(err) => Some(KillBot {
                id,
                reason: format!("{:?}", err),
                killer: None,
            }),
        };

        if let Some(kill) = kill {
            super::kill::run(world, kill);
        }
    }
}
