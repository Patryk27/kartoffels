use crate::{
    AliveBot, BotAction, Bots, Clock, Event, KillBot, Map, Objects, TileKind,
    WorldRng,
};
use bevy_ecs::system::{Commands, Res, ResMut};

pub fn tick(
    mut cmds: Commands,
    clock: Res<Clock>,
    map: Res<Map>,
    mut bots: ResMut<Bots>,
    mut objects: ResMut<Objects>,
    mut rng: ResMut<WorldRng>,
) {
    for _ in 0..clock.ticks() {
        let mut idx = 0;
        let len = bots.alive.len();

        while idx < len {
            if let Some(bot) = bots.alive.begin(idx) {
                let id = bot.id;
                let pos = bot.pos;

                let bot = tick_bot(
                    &mut cmds,
                    &clock,
                    &map,
                    &mut bots,
                    &mut objects,
                    &mut rng,
                    bot,
                );

                bots.alive.commit(idx, id, pos, bot);
            }

            idx += 1;
        }
    }
}

fn tick_bot(
    cmds: &mut Commands,
    clock: &Clock,
    map: &Map,
    bots: &mut Bots,
    objects: &mut Objects,
    rng: &mut WorldRng,
    mut bot: Box<AliveBot>,
) -> Option<Box<AliveBot>> {
    match bot.tick(&bots.alive, map, objects, rng) {
        Ok(Some(BotAction::ArmDrop { at, idx })) => {
            if let Some((id, obj)) = bot.inventory.take(idx) {
                bot.log(
                    clock,
                    format!("dropped {} at {},{}", obj.name(), at.x, at.y),
                );

                cmds.send_event(Event::ObjectDropped { id });
                objects.add(id, obj, Some(at));
            } else {
                bot.log(clock, "dropped nothing");
            }
        }

        Ok(Some(BotAction::ArmPick { at })) => {
            if let Some((id, obj)) = objects.remove_at(at) {
                match bot.inventory.add(id, obj) {
                    Ok(_) => {
                        cmds.send_event(Event::ObjectPicked { id });

                        bot.log(
                            clock,
                            format!(
                                "picked {} from {},{}",
                                obj.name(),
                                at.x,
                                at.y
                            ),
                        );
                    }

                    Err(_) => {
                        bot.log(
                            clock,
                            format!(
                                "failed to pick {} from {},{} (inventory full)",
                                obj.name(),
                                at.x,
                                at.y
                            ),
                        );

                        objects.add(id, obj, Some(at));
                    }
                }
            } else {
                bot.log(clock, "picked fresh air");
            }
        }

        Ok(Some(BotAction::ArmStab { at })) => {
            if let Some(killed) = bots.alive.remove_at(at) {
                bot.log(clock, format!("killed {} (knife)", killed.id));

                cmds.send_event(KillBot {
                    killed: Some(killed),
                    reason: format!("killed by {} (knife)", bot.id),
                    killer: Some(bot.id),
                });
            } else {
                bot.log(clock, "stabbed fresh air");
            }
        }

        Ok(Some(BotAction::MotorMove { at })) => match map.get(at).kind {
            TileKind::VOID => {
                cmds.send_event(KillBot {
                    killed: Some(bot),
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
                killed: Some(bot),
                reason: format!("firmware crashed: {err}"),
                killer: None,
            });

            return None;
        }
    };

    Some(bot)
}
