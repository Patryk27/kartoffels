use crate::*;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BotArm {
    cooldown: u32,
}

impl BotArm {
    pub(super) fn tick(bot: &mut AliveBotBody) {
        bot.arm.cooldown = bot.arm.cooldown.saturating_sub(1);
    }

    pub(super) fn load(bot: &AliveBotBody, addr: u32) -> Result<u32, ()> {
        match addr {
            api::MEM_ARM => Ok((bot.arm.cooldown == 0) as u32),
            _ => Err(()),
        }
    }

    pub(super) fn store(
        bot: &mut AliveBotBody,
        world: &mut World,
        addr: u32,
        val: u32,
    ) -> Result<(), ()> {
        match (addr, val.to_le_bytes()) {
            (api::MEM_ARM, [0x01, 0x00, 0x00, 0x00]) => {
                Self::do_stab(bot, world);
                Ok(())
            }

            (api::MEM_ARM, [0x02, 0x00, 0x00, 0x00]) => {
                Self::do_pick(bot, world);
                Ok(())
            }

            (api::MEM_ARM, [0x03, idx, 0x00, 0x00]) => {
                Self::do_drop(bot, world, idx);
                Ok(())
            }

            _ => Err(()),
        }
    }

    fn do_stab(bot: &mut AliveBotBody, world: &mut World) {
        if bot.arm.cooldown > 0 {
            return;
        }

        let at = bot.pos + bot.dir;

        if let Some(killed) = world.bots.alive.remove_at(at) {
            bot.events
                .add(&world.clock, format!("killed {} (knife)", killed.id));

            world.kill_bot(
                killed,
                format!("killed by {} (knife)", bot.id),
                Some(bot.id),
            );
        } else {
            bot.events.add(&world.clock, "stabbed fresh air");
        }

        bot.arm.cooldown = world.cooldown(60_000);
    }

    fn do_pick(bot: &mut AliveBotBody, world: &mut World) {
        if bot.arm.cooldown > 0 {
            return;
        }

        let at = bot.pos + bot.dir;

        if let Some((id, obj)) = world.objects.remove_at(at) {
            match bot.inventory.add(id, obj) {
                Ok(_) => {
                    world.events.add(Event::ObjectPicked { id });

                    bot.events.add(
                        &world.clock,
                        format!("picked {} from {},{}", obj.name(), at.x, at.y),
                    );
                }

                Err(_) => {
                    bot.events.add(
                        &world.clock,
                        format!(
                            "failed to pick {} from {},{} (inventory full)",
                            obj.name(),
                            at.x,
                            at.y
                        ),
                    );

                    world.objects.add(id, obj, at);
                }
            }
        } else {
            bot.events.add(&world.clock, "picked fresh air");
        }

        bot.arm.cooldown = world.cooldown(60_000);
    }

    fn do_drop(bot: &mut AliveBotBody, world: &mut World, idx: u8) {
        if bot.arm.cooldown > 0 {
            return;
        }

        let at = bot.pos + bot.dir;

        if let Some((id, obj)) = bot.inventory.take(idx) {
            bot.events.add(
                &world.clock,
                format!("dropped {} at {},{}", obj.name(), at.x, at.y),
            );

            world.events.add(Event::ObjectDropped { id });
            world.objects.add(id, obj, at);
        } else {
            bot.events.add(&world.clock, "dropped nothing");
        }

        bot.arm.cooldown = world.cooldown(60_000);
    }
}
