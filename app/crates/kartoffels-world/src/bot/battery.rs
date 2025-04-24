use anyhow::Result;
use kartoffel::MEM_BATTERY;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BotBattery {
    energy: u32,
}

impl BotBattery {
    pub fn mmio_load(&self, addr: u32) -> Result<u32, ()> {
        match addr {
            MEM_BATTERY => Ok(self.energy),

            _ => Err(()),
        }
    }

    pub fn mmio_store(&mut self, _addr: u32, _val: u32) -> Result<(), ()> {
        Err(())
    }
}

impl Default for BotBattery {
    fn default() -> Self {
        Self { energy: 4096 }
    }
}
