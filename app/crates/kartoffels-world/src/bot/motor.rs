use crate::*;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BotMotor {
    cooldown: u32,
}

impl BotMotor {
    pub(super) fn tick(bot: &mut AliveBotBody) {
        bot.motor.cooldown = bot.motor.cooldown.saturating_sub(1);
    }

    pub(super) fn load(bot: &AliveBotBody, addr: u32) -> Result<u32, ()> {
        match addr {
            api::MEM_MOTOR => Ok((bot.motor.cooldown == 0) as u32),
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
            (api::MEM_MOTOR, [0x01, 0x01, 0x01, 0x00]) => {
                Self::do_move(bot, world, BotMotorOp::StepFw);
                Ok(())
            }

            (api::MEM_MOTOR, [0x01, 0xff, 0xff, 0x00]) => {
                Self::do_move(bot, world, BotMotorOp::StepBw);
                Ok(())
            }

            (api::MEM_MOTOR, [0x01, 0x01, 0xff, 0x00]) => {
                Self::do_move(bot, world, BotMotorOp::TurnRight);
                Ok(())
            }

            (api::MEM_MOTOR, [0x01, 0xff, 0x01, 0x00]) => {
                Self::do_move(bot, world, BotMotorOp::TurnLeft);
                Ok(())
            }

            _ => Err(()),
        }
    }

    fn do_move(bot: &mut AliveBotBody, world: &mut World, op: BotMotorOp) {
        if bot.motor.cooldown > 0 {
            return;
        }

        match op {
            BotMotorOp::StepFw | BotMotorOp::StepBw => {
                let at = match op {
                    BotMotorOp::StepFw => bot.pos + bot.dir,
                    _ => bot.pos + bot.dir.turned_back(),
                };

                match world.map.get(at).kind {
                    TileKind::VOID => {
                        bot.pos = AliveBotBody::FELL_INTO_VOID;
                    }

                    TileKind::FLOOR => {
                        if world.bots.alive.lookup_at(at).is_none()
                            && world.objects.lookup_at(at).is_none()
                        {
                            bot.pos = at;

                            world
                                .events
                                .add(Event::BotMoved { id: bot.id, at });
                        }
                    }

                    _ => (),
                }
            }

            BotMotorOp::TurnLeft => {
                bot.dir = bot.dir.turned_left();
            }
            BotMotorOp::TurnRight => {
                bot.dir = bot.dir.turned_right();
            }
        }

        bot.motor.cooldown = world.cooldown(match op {
            BotMotorOp::StepFw => 20_000,
            BotMotorOp::StepBw => 30_000,
            _ => 25_000,
        });
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BotMotorOp {
    StepFw,
    StepBw,
    TurnLeft,
    TurnRight,
}
