use crate::{
    AliveBot, BotAction, Bots, Event, KillBot, Map, Objects, TileKind, WorldRng,
};
use bevy_ecs::system::{Commands, Res, ResMut};
use itertools::Either;

pub fn tick(
    mut cmds: Commands,
    map: Res<Map>,
    mut bots: ResMut<Bots>,
    mut objects: ResMut<Objects>,
    mut rng: ResMut<WorldRng>,
) {
    let mut idx = 0;
    let len = bots.alive.len();

    while idx < len {
        if let Some(bot) = bots.alive.take(idx) {
            let id = bot.id;
            let pos = bot.pos;

            let bot = tick_bot(
                &mut cmds,
                &map,
                &mut bots,
                &mut objects,
                &mut rng,
                bot,
            );

            bots.alive.insert(idx, id, pos, bot);
        }

        idx += 1;
    }
}

fn tick_bot(
    cmds: &mut Commands,
    map: &Map,
    bots: &mut Bots,
    objects: &mut Objects,
    rng: &mut WorldRng,
    mut bot: Box<AliveBot>,
) -> Option<Box<AliveBot>> {
    match bot.tick(&bots.alive, map, objects, rng) {
        Ok(Some(BotAction::ArmDrop { at, idx })) => {
            if let Some((id, obj)) = bot.inventory.take(idx) {
                bot.log(format!("dropped {} at {},{}", obj.name(), at.x, at.y));
                cmds.send_event(Event::ObjectDropped { id });
                objects.add(id, obj, Some(at));
            } else {
                bot.log("dropped nothing");
            }
        }

        Ok(Some(BotAction::ArmPick { at })) => {
            if let Some((id, obj)) = objects.remove_at(at) {
                match bot.inventory.add(id, obj) {
                    Ok(_) => {
                        cmds.send_event(Event::ObjectPicked { id });

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

                        objects.add(id, obj, Some(at));
                    }
                }
            } else {
                bot.log("picked fresh air");
            }
        }

        Ok(Some(BotAction::ArmStab { at })) => {
            if let Some(killed_id) = bots.alive.lookup_at(at) {
                bot.log(format!("killed {killed_id} (knife)"));

                cmds.send_event(KillBot {
                    killed: Either::Left(killed_id),
                    reason: format!("killed by {} (knife)", bot.id),
                    killer: Some(bot.id),
                });
            } else {
                bot.log("stabbed fresh air");
            }
        }

        Ok(Some(BotAction::MotorMove { at })) => match map.get(at).kind {
            TileKind::VOID => {
                cmds.send_event(KillBot {
                    killed: Either::Right(Some(bot)),
                    reason: "fell into the void".into(),
                    killer: None,
                });

                return None;
            }

            TileKind::FLOOR => {
                if bots.alive.lookup_at(at).is_none()
                    && objects.lookup_at(at).is_none()
                {
                    bot.pos = at;

                    cmds.send_event(Event::BotMoved { id: bot.id, at });
                }
            }

            _ => (),
        },

        Ok(None) => {
            //
        }

        Err(err) => {
            cmds.send_event(KillBot {
                killed: Either::Right(Some(bot)),
                reason: format!("firmware crashed: {err}"),
                killer: None,
            });

            return None;
        }
    };

    Some(bot)
}
