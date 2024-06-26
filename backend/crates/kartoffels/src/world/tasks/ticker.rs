use crate::{AliveBotEntryMut, World};
use rand::RngCore;

#[derive(Debug)]
pub struct Controller;

impl Controller {
    pub fn tick(&mut self, world: &mut World, rng: &mut impl RngCore) {
        for _ in 0..World::SIM_TICKS {
            self.tick_once(world, rng);
        }

        world
            .mode
            .on_after_tick(rng, &mut world.theme, &mut world.map);
    }

    fn tick_once(&mut self, world: &mut World, rng: &mut impl RngCore) {
        for id in world.bots.alive.pick_ids(rng) {
            let Some((AliveBotEntryMut { pos, bot }, bots)) =
                world.bots.alive.try_get_mut(id)
            else {
                // Our bot got killed in the meantime, happens
                continue;
            };

            match bot.tick(&world.map, &bots, pos) {
                Ok(state) => {
                    state.apply(rng, world, id, pos);
                }

                Err(err) => {
                    world.bots.kill(
                        rng,
                        &mut world.mode,
                        &world.policy,
                        id,
                        &format!("{:?}", err),
                        None,
                    );
                }
            }
        }
    }
}