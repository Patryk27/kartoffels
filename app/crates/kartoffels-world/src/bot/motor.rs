use super::BotAction;
use crate::{AliveBot, BotMmioContext};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BotMotor {
    cooldown: u32,
}

impl BotMotor {
    pub fn tick(&mut self) {
        self.cooldown = self.cooldown.saturating_sub(1);
    }

    pub fn mmio_load(&self, addr: u32) -> Result<u32, ()> {
        match addr {
            AliveBot::MEM_MOTOR => Ok((self.cooldown == 0) as u32),

            _ => Err(()),
        }
    }

    pub fn mmio_store(
        &mut self,
        ctxt: &mut BotMmioContext,
        addr: u32,
        val: u32,
    ) -> Result<(), ()> {
        match (addr, val.to_le_bytes()) {
            (AliveBot::MEM_MOTOR, [0x01, 0x01, 0x01, 0x00]) => {
                if self.cooldown == 0 {
                    *ctxt.action = Some(BotAction::MotorMove {
                        at: ctxt.pos + *ctxt.dir,
                    });

                    self.cooldown = ctxt.cooldown(20_000);
                }

                Ok(())
            }

            (AliveBot::MEM_MOTOR, [0x01, 0xff, 0xff, 0x00]) => {
                if self.cooldown == 0 {
                    *ctxt.action = Some(BotAction::MotorMove {
                        at: ctxt.pos + ctxt.dir.turned_back(),
                    });

                    self.cooldown = ctxt.cooldown(30_000);
                }

                Ok(())
            }

            (AliveBot::MEM_MOTOR, [0x01, 0x01, 0xff, 0x00]) => {
                if self.cooldown == 0 {
                    *ctxt.dir = ctxt.dir.turned_right();

                    self.cooldown = ctxt.cooldown(25_000);
                }

                Ok(())
            }

            (AliveBot::MEM_MOTOR, [0x01, 0xff, 0x01, 0x00]) => {
                if self.cooldown == 0 {
                    *ctxt.dir = ctxt.dir.turned_left();

                    self.cooldown = ctxt.cooldown(25_000);
                }

                Ok(())
            }

            _ => Err(()),
        }
    }
}
