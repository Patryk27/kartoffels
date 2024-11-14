use crate::{AliveBot, BotAction, KillBot, TileKind, World};
use itertools::Either;

pub fn run(world: &mut World) {
    let ticks = world.clock.ticks();

    for _ in 0..ticks {
        world_tick(world);
    }
}

fn world_tick(world: &mut World) {
    let len = world.bots.alive.len();
    let mut idx = 0;

    while idx < len {
        if let Some(bot) = world.bots.alive.take(idx) {
            let id = bot.id;
            let pos = bot.pos;
            let bot = bot_tick(world, bot);

            world.bots.alive.insert(idx, id, pos, bot);
        }

        idx += 1;
    }
}

fn bot_tick(
    world: &mut World,
    mut bot: Box<AliveBot>,
) -> Option<Box<AliveBot>> {
    match bot.tick(world) {
        Ok(Some(BotAction::ArmDrop { at, idx })) => {
            if let Some(object) = bot.inventory.take(idx) {
                bot.log(format!(
                    "dropped {} at {},{}",
                    object.name(),
                    at.x,
                    at.y
                ));

                world.objects.put(at, object);
            }
        }

        Ok(Some(BotAction::ArmPick { at })) => {
            if let Some(object) = world.objects.take(at) {
                match bot.inventory.add(object) {
                    Ok(_) => {
                        bot.log(format!(
                            "picked {} from {},{}",
                            object.name(),
                            at.x,
                            at.y
                        ));
                    }

                    Err(object) => {
                        bot.log(format!(
                            "failed to pick {} from {},{} (inventory full)",
                            object.name(),
                            at.x,
                            at.y
                        ));

                        world.objects.put(at, object);
                    }
                }
            }
        }

        Ok(Some(BotAction::ArmStab { at })) => {
            if let Some(killed_id) = world.bots.alive.get_by_pos(at) {
                bot.log(format!("killed {killed_id} (knife)"));

                let kill = KillBot {
                    killed: Either::Left(killed_id),
                    reason: format!("killed by {} (knife)", bot.id),
                    killer: Some(bot.id),
                };

                super::kill::run(world, kill);
            }
        }

        Ok(Some(BotAction::MotorMove { at })) => match world.map.get(at).kind {
            TileKind::VOID => {
                let kill = KillBot {
                    killed: Either::Right(bot),
                    reason: "fell into the void".into(),
                    killer: None,
                };

                super::kill::run(world, kill);

                return None;
            }

            TileKind::FLOOR => {
                if world.bots.alive.get_by_pos(at).is_none()
                    && world.objects.get(at).is_none()
                {
                    bot.pos = at;
                }
            }

            _ => (),
        },

        Ok(None) => {
            //
        }

        Err(err) => {
            let kill = KillBot {
                killed: Either::Right(bot),
                reason: format!("firmware crashed: {err}"),
                killer: None,
            };

            super::kill::run(world, kill);

            return None;
        }
    };

    Some(bot)
}
