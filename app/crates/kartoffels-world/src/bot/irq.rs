use super::{AliveBot, AliveBotBody};
use kartoffel as api;
use kartoffels_cpu::Atomic;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BotIrq {
    memory: Vec<u32>,
    args: Vec<u32>,
    pending: u32,
}

impl BotIrq {
    pub fn raise(&mut self, idx: u8, arg: [u8; 3]) {
        let idx = idx as usize;

        self.args[idx] =
            u32::from_le_bytes([idx as u8, arg[0], arg[1], arg[2]]);

        self.pending |= 1 << idx;
    }

    pub fn take(&mut self) -> Option<u32> {
        if self.pending == 0 {
            return None;
        }

        let idx = self.pending.trailing_zeros() as usize;
        let arg = self.args[idx];

        self.pending &= !(1 << idx);

        Some(arg)
    }

    #[cfg(test)]
    pub fn take_le(&mut self) -> Option<[u8; 4]> {
        Some(self.take()?.to_le_bytes())
    }

    pub(super) fn tick(bot: &mut AliveBot) {
        if bot.cpu.is_executing_irq() {
            return;
        }

        let Some(arg) = bot.irq.take() else {
            return;
        };

        let idx = arg.to_le_bytes()[0];
        let pc = bot.irq.memory[1 + idx as usize];

        if pc > 0 {
            bot.cpu.irq(pc, arg);
        }
    }

    pub(super) fn load(bot: &AliveBotBody, addr: u32) -> Result<u32, ()> {
        if (api::IRQ_MEM + 4..api::SERIAL_MEM).contains(&addr) {
            Ok(bot.irq.memory[(addr - api::IRQ_MEM) as usize / 4])
        } else {
            Err(())
        }
    }

    // TODO start, stop, restart?
    pub(super) fn store(
        bot: &mut AliveBotBody,
        addr: u32,
        val: u32,
    ) -> Result<(), ()> {
        if (api::IRQ_MEM + 4..api::SERIAL_MEM).contains(&addr) {
            bot.irq.memory[(addr - api::IRQ_MEM) as usize / 4] = val;
            Ok(())
        } else {
            Err(())
        }
    }

    pub(super) fn atomic(
        bot: &mut AliveBotBody,
        addr: u32,
        rhs: u32,
        op: Atomic,
    ) -> Result<u32, ()> {
        let lhs = Self::load(bot, addr)?;
        let out = op.eval(lhs, rhs);

        Self::store(bot, addr, out)?;

        Ok(lhs)
    }
}

impl Default for BotIrq {
    fn default() -> Self {
        Self {
            memory: vec![0; 256],
            args: vec![0; 32],
            pending: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let mut target = BotIrq::default();

        // ---

        assert_eq!(None, target.take());

        // ---

        target.raise(0x00, [0xaa, 0xaa, 0xaa]);
        target.raise(0x01, [0xbb, 0xbb, 0xbb]);
        target.raise(0x02, [0xcc, 0xcc, 0xcc]);

        assert_eq!([0, 0xaa, 0xaa, 0xaa], target.take().unwrap().to_le_bytes());
        assert_eq!([1, 0xbb, 0xbb, 0xbb], target.take().unwrap().to_le_bytes());
        assert_eq!([2, 0xcc, 0xcc, 0xcc], target.take().unwrap().to_le_bytes());
        assert_eq!(None, target.take());

        // ---

        target.raise(0x02, [0xcc, 0xcc, 0xcc]);
        target.raise(0x00, [0xaa, 0xaa, 0xaa]);
        target.raise(0x01, [0xbb, 0xbb, 0xbb]);

        assert_eq!([0, 0xaa, 0xaa, 0xaa], target.take().unwrap().to_le_bytes());
        assert_eq!([1, 0xbb, 0xbb, 0xbb], target.take().unwrap().to_le_bytes());
        assert_eq!([2, 0xcc, 0xcc, 0xcc], target.take().unwrap().to_le_bytes());
        assert_eq!(None, target.take());

        // ---

        for i in 0..32 {
            target.raise(i, [3 * i, 5 * i, 7 * i]);
        }

        for i in 0..32 {
            assert_eq!(
                [i, 3 * i, 5 * i, 7 * i],
                target.take().unwrap().to_le_bytes()
            );
        }

        assert_eq!(None, target.take());
    }
}
