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
            if let Some((id, obj)) = bot.inventory.take(idx) {
                bot.log(format!("dropped {} at {},{}", obj.name(), at.x, at.y));

                world.objects.add(id, obj, Some(at));
            } else {
                bot.log("dropped nothing");
            }
        }

        Ok(Some(BotAction::ArmPick { at })) => {
            if let Some((id, obj)) = world.objects.remove_at(at) {
                match bot.inventory.add(id, obj) {
                    Ok(_) => {
                        bot.log(format!(
                            "picked {} from {},{}",
                            obj.name(),
                            at.x,
                            at.y
                        ));
                    }

                    Err(_) => {
                        bot.log(format!(
                            "failed to pick {} from {},{} (inventory full)",
                            obj.name(),
                            at.x,
                            at.y
                        ));

                        world.objects.add(id, obj, Some(at));
                    }
                }
            } else {
                bot.log("picked fresh air");
            }
        }

        Ok(Some(BotAction::ArmStab { at })) => {
            if let Some(killed_id) = world.bots.alive.lookup_at(at) {
                bot.log(format!("killed {killed_id} (knife)"));

                let kill = KillBot {
                    killed: Either::Left(killed_id),
                    reason: format!("killed by {} (knife)", bot.id),
                    killer: Some(bot.id),
                };

                super::kill::run(world, kill);
            } else {
                bot.log("stabbed fresh air");
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
                if world.bots.alive.lookup_at(at).is_none()
                    && world.objects.lookup_at(at).is_none()
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
