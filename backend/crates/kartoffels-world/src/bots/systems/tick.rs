use crate::{AliveBotEntryMut, BotAction, BotId, KillBot, TileKind, World};

pub fn run(world: &mut World) {
    let ids = world.bots.alive.ids();
    let steps = world.clock.steps();

    for _ in 0..steps {
        for id in &ids {
            tick(world, *id);
        }
    }
}

fn tick(world: &mut World, id: BotId) {
    let Some(AliveBotEntryMut { bot, locator }) = world.bots.alive.get_mut(id)
    else {
        // Our bot got killed in the meantime, happens
        return;
    };

    match bot.tick(&mut world.rng, &world.map, &locator) {
        Ok(Some(BotAction::ArmDrop { at, idx })) => {
            if let Some(object) = bot.inventory.take(idx) {
                world.objects.put(at, object);
            }
        }

        Ok(Some(BotAction::ArmPick { at })) => {
            if let Some(object) = world.objects.take(at) {
                if let Err(object) = bot.inventory.add(object) {
                    world.objects.put(at, object);
                }
            }
        }

        Ok(Some(BotAction::ArmStab { at })) => {
            if let Some(killed_id) = world.bots.alive.get_by_pos(at) {
                let kill = KillBot {
                    id: killed_id,
                    reason: format!("stabbed out of existence by {id}"),
                    killer: Some(id),
                };

                super::kill::run(world, kill);
            }
        }

        Ok(Some(BotAction::MotorMove { at })) => match world.map.get(at).kind {
            TileKind::VOID => {
                let kill = KillBot {
                    id,
                    reason: "fell into the void".into(),
                    killer: None,
                };

                super::kill::run(world, kill);
            }

            TileKind::FLOOR => {
                if world.bots.alive.get_by_pos(at).is_none() {
                    world.bots.alive.relocate(id, at);
                }
            }

            _ => (),
        },

        Ok(None) => {
            //
        }

        Err(err) => {
            let kill = KillBot {
                id,
                reason: format!("firmware crashed: {err}"),
                killer: None,
            };

            super::kill::run(world, kill);
        }
    };
}
