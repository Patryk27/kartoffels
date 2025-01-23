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
        match addr {
            AliveBot::MEM_MOTOR => {
                if self.cooldown == 0 && val > 0 {
                    *ctxt.action = Some(BotAction::MotorMove {
                        at: ctxt.pos + *ctxt.dir,
                    });

                    self.cooldown = ctxt.cooldown(20_000, 15);
                }

                Ok(())
            }

            const { AliveBot::MEM_MOTOR + 4 } => {
                if self.cooldown == 0 {
                    let val = val as i32;

                    #[allow(clippy::comparison_chain)]
                    if val < 0 {
                        *ctxt.dir = ctxt.dir.turned_left();
                    } else if val > 0 {
                        *ctxt.dir = ctxt.dir.turned_right();
                    }

                    if val != 0 {
                        self.cooldown = ctxt.cooldown(15000, 15);
                    }
                }

                Ok(())
            }

            _ => Err(()),
        }
    }
}
