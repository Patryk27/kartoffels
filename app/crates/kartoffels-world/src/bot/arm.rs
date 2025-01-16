use super::BotAction;
use crate::{AliveBot, BotMmioContext};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BotArm {
    cooldown: u32,
}

impl BotArm {
    pub fn tick(&mut self) {
        self.cooldown = self.cooldown.saturating_sub(1);
    }

    pub fn mmio_load(&self, addr: u32) -> Result<u32, ()> {
        match addr {
            AliveBot::MEM_ARM => Ok((self.cooldown == 0) as u32),

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
            AliveBot::MEM_ARM => {
                if self.cooldown == 0 {
                    match val.to_be_bytes() {
                        [0, 0, 0, 1] => {
                            *ctxt.action = Some(BotAction::ArmStab {
                                at: ctxt.pos + *ctxt.dir,
                            });

                            self.cooldown = ctxt.cooldown(60_000, 15);
                        }

                        [0, 0, 0, 2] => {
                            *ctxt.action = Some(BotAction::ArmPick {
                                at: ctxt.pos + *ctxt.dir,
                            });

                            self.cooldown = ctxt.cooldown(60_000, 15);
                        }

                        [0, 0, idx, 3] => {
                            *ctxt.action = Some(BotAction::ArmDrop {
                                at: ctxt.pos + *ctxt.dir,
                                idx,
                            });

                            self.cooldown = ctxt.cooldown(60_000, 15);
                        }

                        _ => (),
                    }
                }

                Ok(())
            }

            _ => Err(()),
        }
    }
}
