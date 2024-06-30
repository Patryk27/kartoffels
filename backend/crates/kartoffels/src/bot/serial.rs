use crate::AliveBot;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BotSerial {
    pub buffer: VecDeque<u32>,
}

impl BotSerial {
    const CAPACITY: usize = 256;

    pub fn tick(&mut self) {
        // no-op
    }

    pub fn mmio_load(&self, _addr: u32) -> Result<u32, ()> {
        Err(())
    }

    pub fn mmio_store(&mut self, addr: u32, val: u32) -> Result<(), ()> {
        match addr {
            AliveBot::MEM_SERIAL => {
                if self.buffer.len() >= Self::CAPACITY {
                    self.buffer.pop_front();
                }

                self.buffer.push_back(val);

                Ok(())
            }

            _ => Err(()),
        }
    }
}
