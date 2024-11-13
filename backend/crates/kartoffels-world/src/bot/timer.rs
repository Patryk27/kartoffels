use crate::{AliveBot, Clock};
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default))]
pub struct BotTimer {
    seed: u32,
    ticks: u32, // TOOD overflows after ~18h
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

    pub fn age(&self, clock: &Clock) -> u32 {
        match clock {
            Clock::Auto => self.ticks / Clock::HZ,
            Clock::Manual { ticks } => self.ticks / ticks,
        }
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
