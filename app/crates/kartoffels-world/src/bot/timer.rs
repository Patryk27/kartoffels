use kartoffel::MEM_TIMER;
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default))]
pub struct BotTimer {
    seed: u32,
    ticks: u64,
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

    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    pub fn mmio_load(&self, addr: u32) -> Result<u32, ()> {
        match addr {
            MEM_TIMER => Ok(self.seed),
            const { MEM_TIMER + 4 } => Ok(self.ticks as u32),

            _ => Err(()),
        }
    }

    pub fn mmio_store(&mut self, _addr: u32, _val: u32) -> Result<(), ()> {
        Err(())
    }
}
