use crate::{AliveBot, BotMmioContext};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BotArm {
    pub cooldown: u32,
    pub is_stabbing: bool,
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
                if self.cooldown == 0 && val > 0 {
                    self.is_stabbing = true;
                    self.cooldown = ctxt.cooldown(60_000, 15);
                }

                Ok(())
            }

            _ => Err(()),
        }
    }
}
