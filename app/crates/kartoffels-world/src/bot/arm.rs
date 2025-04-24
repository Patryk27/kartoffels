use super::BotAction;
use crate::BotMmioContext;
use anyhow::Result;
use kartoffel::MEM_ARM;
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
            MEM_ARM => Ok((self.cooldown == 0) as u32),

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
            (MEM_ARM, [0x01, 0x00, 0x00, 0x00]) => {
                if self.cooldown == 0 {
                    *ctxt.action = Some(BotAction::ArmStab {
                        at: ctxt.pos + *ctxt.dir,
                    });

                    self.cooldown = ctxt.cooldown(60_000);
                }

                Ok(())
            }

            (MEM_ARM, [0x02, 0x00, 0x00, 0x00]) => {
                if self.cooldown == 0 {
                    *ctxt.action = Some(BotAction::ArmPick {
                        at: ctxt.pos + *ctxt.dir,
                    });

                    self.cooldown = ctxt.cooldown(60_000);
                }

                Ok(())
            }

            (MEM_ARM, [0x03, idx, 0x00, 0x00]) => {
                if self.cooldown == 0 {
                    *ctxt.action = Some(BotAction::ArmDrop {
                        at: ctxt.pos + *ctxt.dir,
                        idx,
                    });

                    self.cooldown = ctxt.cooldown(60_000);
                }

                Ok(())
            }

            _ => Err(()),
        }
    }
}
