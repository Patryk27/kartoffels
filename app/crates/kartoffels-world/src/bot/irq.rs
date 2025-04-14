use crate::AliveBot;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BotIrq {
    memory: Vec<u32>,
    pending: u32,
    enabled: bool,
}

impl BotIrq {
    pub fn raise(&mut self, irq: u8) {
        self.pending |= 1 << irq;
    }

    pub fn lower(&mut self, irq: u8) {
        self.pending &= !(1 << irq);
    }

    pub fn mmio_load(&self, addr: u32) -> Result<u32, ()> {
        if addr >= AliveBot::MEM_IRQ && addr < AliveBot::MEM_SERIAL {
            Ok(self.memory[(addr - AliveBot::MEM_IRQ) as usize / 4])
        } else {
            Err(())
        }
    }

    pub fn mmio_store(&mut self, addr: u32, val: u32) -> Result<(), ()> {
        if addr == AliveBot::MEM_IRQ {
            match val.to_le_bytes() {
                [0x01, 0x00, 0x00, 0x00] => {
                    self.enabled = false;
                    Ok(())
                }
                [0x01, 0x01, 0x00, 0x00] => {
                    self.enabled = true;
                    Ok(())
                }
                _ => Err(()),
            }
        } else if addr > AliveBot::MEM_IRQ && addr < AliveBot::MEM_SERIAL {
            self.memory[(addr - AliveBot::MEM_IRQ) as usize / 4] = val;

            Ok(())
        } else {
            Err(())
        }
    }
}

impl Default for BotIrq {
    fn default() -> Self {
        Self {
            memory: Vec::with_capacity(256),
            enabled: false,
            pending: 0,
        }
    }
}
