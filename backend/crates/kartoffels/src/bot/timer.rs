use crate::{AliveBot, World};
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default))]
pub struct BotTimer {
    pub seed: u32,
    pub ticks: u32, // TOOD overflows after ~18h
}

impl BotTimer {
    pub fn new(rng: &mut impl RngCore) -> Self {
        Self {
            seed: rng.gen(),
            ticks: 0,
        }
    }

    pub fn tick(&mut self) {
        self.ticks += 1;
    }

    pub fn age(&self) -> f32 {
        (self.ticks as f32) / (World::SIM_HZ as f32)
    }

    pub fn mmio_load(&self, addr: u32) -> Result<u32, ()> {
        match addr {
            AliveBot::MEM_TIMER => Ok(self.seed),
            const { AliveBot::MEM_TIMER + 4 } => Ok(self.ticks),

            _ => Err(()),
        }
    }

    pub fn mmio_store(&mut self, _addr: u32, _val: u32) -> Result<(), ()> {
        Err(())
    }
}
