use crate::AliveBot;
use anyhow::Result;
use kartoffels_cpu::Cpu;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BotIrq {
    memory: Vec<u32>,
    args: Vec<u32>,
    flags: u32,
    enabled: bool,
}

impl BotIrq {
    pub fn raise(&mut self, irq: u8, arg: [u8; 3]) {
        assert!((irq as u32) < u32::BITS);

        let irq = irq as usize;

        self.args.resize(self.args.len().max(irq + 1), 0);

        self.args[irq] =
            u32::from_le_bytes([irq as u8, arg[0], arg[1], arg[2]]);

        self.flags |= 1 << irq;
    }

    fn take(&mut self) -> Option<(u32, u32)> {
        if !self.enabled || self.flags == 0 {
            return None;
        }

        let irq = self.flags.trailing_zeros() as usize;
        let pc = self.memory[1 + irq];
        let arg = self.args[irq];

        self.flags &= !(1 << irq);

        Some((pc, arg))
    }

    pub fn tick(&mut self, cpu: &mut Cpu) {
        if cpu.is_executing_irq() {
            return;
        }

        let Some((pc, arg)) = self.take() else {
            return;
        };

        if pc > 0 {
            cpu.irq(pc, arg);
        }
    }

    pub fn mmio_load(&self, addr: u32) -> Result<u32, ()> {
        if (AliveBot::MEM_IRQ..AliveBot::MEM_SERIAL).contains(&addr) {
            Ok(self.memory[(addr - AliveBot::MEM_IRQ) as usize / 4])
        } else {
            Err(())
        }
    }

    pub fn mmio_store(&mut self, addr: u32, val: u32) -> Result<(), ()> {
        match (addr, val.to_le_bytes()) {
            (AliveBot::MEM_IRQ, [0x01, 0x00, 0x00, 0x00]) => {
                self.enabled = false;

                Ok(())
            }

            (AliveBot::MEM_IRQ, [0x01, 0x01, 0x00, 0x00]) => {
                self.enabled = true;

                Ok(())
            }

            (AliveBot::MEM_IRQ, [0x01, 0x02, 0x00, 0x00]) => {
                self.flags = 0;
                self.enabled = true;

                Ok(())
            }

            (const { AliveBot::MEM_IRQ + 4 }..AliveBot::MEM_SERIAL, _) => {
                self.memory[(addr - AliveBot::MEM_IRQ) as usize / 4] = val;

                Ok(())
            }

            _ => Err(()),
        }
    }
}

impl Default for BotIrq {
    fn default() -> Self {
        Self {
            memory: vec![0; 256],
            args: Vec::new(),
            flags: 0,
            enabled: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arg(irq: u8, a1: u8, a2: u8, a3: u8) -> u32 {
        u32::from_le_bytes([irq, a1, a2, a3])
    }

    #[test]
    fn smoke() {
        let mut target = BotIrq::default();

        // ---

        assert_eq!(None, target.take());

        target.enabled = true;

        assert_eq!(None, target.take());

        // ---

        target.raise(0x00, [0xaa, 0xaa, 0xaa]);
        target.raise(0x01, [0xbb, 0xbb, 0xbb]);
        target.raise(0x02, [0xcc, 0xcc, 0xcc]);

        assert_eq!(Some((0, arg(0, 0xaa, 0xaa, 0xaa))), target.take());
        assert_eq!(Some((0, arg(1, 0xbb, 0xbb, 0xbb))), target.take());
        assert_eq!(Some((0, arg(2, 0xcc, 0xcc, 0xcc))), target.take());
        assert_eq!(None, target.take());

        // ---

        target.raise(0x02, [0xcc, 0xcc, 0xcc]);
        target.raise(0x00, [0xaa, 0xaa, 0xaa]);
        target.raise(0x01, [0xbb, 0xbb, 0xbb]);

        assert_eq!(Some((0, arg(0, 0xaa, 0xaa, 0xaa))), target.take());
        assert_eq!(Some((0, arg(1, 0xbb, 0xbb, 0xbb))), target.take());
        assert_eq!(Some((0, arg(2, 0xcc, 0xcc, 0xcc))), target.take());
        assert_eq!(None, target.take());

        // ---

        for irq in 0..32 {
            target.raise(irq, [3 * irq, 5 * irq, 7 * irq]);
        }

        for irq in 0..32 {
            assert_eq!(
                Some((0, arg(irq, 3 * irq, 5 * irq, 7 * irq))),
                target.take()
            );
        }

        assert_eq!(None, target.take());
    }
}
