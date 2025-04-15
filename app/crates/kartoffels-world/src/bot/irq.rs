use crate::AliveBot;
use anyhow::Result;
use kartoffel::{IRQ_FALLING, IRQ_LEVEL, IRQ_RISING};
use kartoffels_cpu::Cpu;
use serde::{Deserialize, Serialize};
use std::mem;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BotIrq {
    memory: Vec<u32>,
    active: Vec<u32>,
    enabled: bool,
}

impl BotIrq {
    pub fn raise(&mut self, irq: u8, arg: u32) {
        let irq = (3 * irq) as usize;
        let arg = arg | 1; // TODO

        self.active[irq + IRQ_LEVEL as usize] = arg;
        self.active[irq + IRQ_RISING as usize] = arg;
        self.active[irq + IRQ_FALLING as usize] = 0;
    }

    pub fn lower(&mut self, irq: u8) {
        let irq = (3 * irq) as usize;
        let arg = mem::take(&mut self.active[irq + IRQ_LEVEL as usize]);

        if arg > 0 {
            self.active[irq + IRQ_RISING as usize] = 0;
            self.active[irq + IRQ_FALLING as usize] = arg;
        }
    }

    pub fn tick(&mut self, cpu: &mut Cpu) {
        if !self.enabled || cpu.is_executing_irq() {
            return;
        }

        for (irq, arg) in self.active.iter_mut().enumerate() {
            if *arg > 0 {
                let pc = self.memory[1 + irq];

                if pc > 0 {
                    cpu.irq(pc, *arg);

                    if irq % 3 > 0 {
                        *arg = 0;
                    }

                    return;
                }
            }
        }
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
                [0x02, _irq, 0x00, 0x0] => {
                    todo!();
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
            memory: vec![0; 256],
            active: vec![0; 256],
            enabled: false,
        }
    }
}
