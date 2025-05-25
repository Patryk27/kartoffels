use super::{AliveBotBody, BotIrq};
use kartoffel as api;
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default))]
pub struct BotClock {
    seed: u32,
    ticks: u64,
    timer0: BotTimer,
    timer1: BotTimer,
    timer2: BotTimer,
}

impl BotClock {
    pub fn new(rng: &mut impl RngCore) -> Self {
        Self {
            seed: rng.r#gen(),
            ticks: 0,
            timer0: BotTimer::default(),
            timer1: BotTimer::default(),
            timer2: BotTimer::default(),
        }
    }

    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    pub(super) fn tick(bot: &mut AliveBotBody) {
        bot.clock.ticks += 1;

        if bot.clock.timer0.is_active() {
            bot.clock.timer0.tick(&mut bot.irq, api::IRQ_TIMER0);
        }

        if bot.clock.timer1.is_active() {
            bot.clock.timer1.tick(&mut bot.irq, api::IRQ_TIMER1);
        }

        if bot.clock.timer2.is_active() {
            bot.clock.timer2.tick(&mut bot.irq, api::IRQ_TIMER2);
        }
    }

    pub(super) fn load(bot: &AliveBotBody, addr: u32) -> Result<u32, ()> {
        match addr {
            // Subregion: clock
            api::CLOCK_MEM => Ok(bot.clock.seed),
            addr if addr == api::CLOCK_MEM + 4 => Ok(bot.clock.ticks as u32),

            // Subregion: timer0
            addr if addr == api::CLOCK_MEM + 10 * 4 => {
                Ok(bot.clock.timer0.load_lo())
            }
            addr if addr == api::CLOCK_MEM + 11 * 4 => {
                Ok(bot.clock.timer0.load_hi())
            }

            // Subregion: timer1
            addr if addr == api::CLOCK_MEM + 12 * 4 => {
                Ok(bot.clock.timer1.load_lo())
            }
            addr if addr == api::CLOCK_MEM + 13 * 4 => {
                Ok(bot.clock.timer1.load_hi())
            }

            // Subregion: timer1
            addr if addr == api::CLOCK_MEM + 14 * 4 => {
                Ok(bot.clock.timer2.load_lo())
            }
            addr if addr == api::CLOCK_MEM + 15 * 4 => {
                Ok(bot.clock.timer2.load_hi())
            }

            _ => Err(()),
        }
    }

    pub(super) fn store(
        bot: &mut AliveBotBody,
        addr: u32,
        val: u32,
    ) -> Result<(), ()> {
        match (addr, val.to_le_bytes()) {
            (addr, _) if addr == api::CLOCK_MEM + 10 * 4 => {
                bot.clock.timer0.store(val);
                Ok(())
            }
            (addr, _) if addr == api::CLOCK_MEM + 12 * 4 => {
                bot.clock.timer1.store(val);
                Ok(())
            }
            (addr, _) if addr == api::CLOCK_MEM + 14 * 4 => {
                bot.clock.timer2.store(val);
                Ok(())
            }

            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
struct BotTimer {
    cfg: u8,
    acc: u16,
    max: u16,
    ticks: u64,
}

impl BotTimer {
    fn is_active(&self) -> bool {
        self.cfg > 0
    }

    #[cold]
    fn tick(&mut self, irq: &mut BotIrq, irq_idx: u8) {
        self.ticks += 1;

        if self.ticks % self.prescaler() != 0 {
            return;
        }

        if self.acc == 0 {
            if self.cfg & api::TIMER_ONESHOT > 0 {
                self.cfg = 0;
            } else {
                self.acc = self.max;
            }

            irq.raise(irq_idx, [self.cfg, 0x00, 0x00]);
        } else {
            self.acc -= 1;
        }
    }

    fn load_lo(&self) -> u32 {
        u32::from_le_bytes([
            self.cfg,
            self.acc.to_le_bytes()[0],
            self.max.to_le_bytes()[0],
            self.max.to_le_bytes()[1],
        ])
    }

    fn load_hi(&self) -> u32 {
        let [acc_lo, acc_hi] = self.acc.to_le_bytes();
        let [max_lo, max_hi] = self.max.to_le_bytes();

        u32::from_le_bytes([acc_lo, acc_hi, max_lo, max_hi])
    }

    fn store(&mut self, val: u32) {
        let [cfg, acc_lo, max_lo, max_hi] = val.to_le_bytes();

        self.cfg = cfg;
        self.ticks = 0;

        if self.cfg & api::TIMER_ONESHOT > 0 {
            self.acc = u16::from_le_bytes([max_lo, max_hi]);
            self.max = self.acc;
        } else {
            self.acc = u16::from_le_bytes([acc_lo, 0]);
            self.max = u16::from_le_bytes([max_lo, max_hi]);
        }
    }

    fn prescaler(&self) -> u64 {
        match self.cfg & 0b111 {
            0 => 0,
            n => 1 << (n + 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AliveBot, World};
    use pretty_assertions as pa;

    #[test]
    fn seed() {
        let mut bot = AliveBot::default();

        bot.clock.seed = 1234;

        assert_eq!(Ok(1234), bot.load(api::CLOCK_MEM));
    }

    #[test]
    fn timer() {
        for (cfg, ps) in [
            (api::TIMER_OFF, 0),
            (api::TIMER_PS_4, 4),
            (api::TIMER_PS_8, 8),
            (api::TIMER_PS_16, 16),
            (api::TIMER_PS_32, 32),
            (api::TIMER_PS_64, 64),
            (api::TIMER_PS_128, 128),
            (api::TIMER_PS_256, 256),
        ] {
            let mut target = BotTimer::default();

            target.store(api::pack(cfg, 25, 50, 75));

            assert_eq!(cfg, target.cfg);
            assert_eq!(25, target.acc);
            assert_eq!(u16::from_le_bytes([50, 75]), target.max);
            assert_eq!(ps, target.prescaler());
        }
    }

    #[test]
    fn timers() {
        let mut bot = AliveBot::default();
        let mut world = World::default();

        bot.store(
            &mut world,
            api::CLOCK_MEM + 10 * 4,
            api::pack(api::TIMER_PS_256 | api::TIMER_ONESHOT, 0, 250, 0),
        )
        .unwrap();

        bot.store(
            &mut world,
            api::CLOCK_MEM + 12 * 4,
            api::pack(api::TIMER_PS_256, 10, 125, 0),
        )
        .unwrap();

        bot.store(
            &mut world,
            api::CLOCK_MEM + 14 * 4,
            api::pack(api::TIMER_PS_256, 20, 62, 0),
        )
        .unwrap();

        // ---

        assert_eq!(
            api::TIMER_PS_256 | api::TIMER_ONESHOT,
            bot.clock.timer0.cfg
        );
        assert_eq!(250, bot.clock.timer0.acc);
        assert_eq!(250, bot.clock.timer0.max);

        assert_eq!(api::TIMER_PS_256, bot.clock.timer1.cfg);
        assert_eq!(10, bot.clock.timer1.acc);
        assert_eq!(125, bot.clock.timer1.max);

        assert_eq!(api::TIMER_PS_256, bot.clock.timer2.cfg);
        assert_eq!(20, bot.clock.timer2.acc);
        assert_eq!(62, bot.clock.timer2.max);

        // ---

        let mut actual = Vec::new();

        for nth in 0..100000 {
            BotClock::tick(&mut bot);

            while let Some([irq, ..]) = bot.irq.take_le() {
                actual.push((nth, irq));
            }
        }

        let expected = vec![
            // (tick, irq)
            (2815, 1),
            (5375, 2),
            (21503, 2),
            (35071, 1),
            (37631, 2),
            (53759, 2),
            (64255, 0),
            (67327, 1),
            (69887, 2),
            (86015, 2),
            (99583, 1),
        ];

        pa::assert_eq!(expected, actual);

        // ---

        assert_eq!(
            [api::TIMER_OFF, 0, 250, 0],
            bot.load(api::CLOCK_MEM + 10 * 4).unwrap().to_le_bytes(),
        );
        assert_eq!(
            [0, 0, 250, 0],
            bot.load(api::CLOCK_MEM + 11 * 4).unwrap().to_le_bytes(),
        );

        assert_eq!(
            [api::TIMER_PS_256, 124, 125, 0],
            bot.load(api::CLOCK_MEM + 12 * 4).unwrap().to_le_bytes(),
        );
        assert_eq!(
            [124, 0, 125, 0],
            bot.load(api::CLOCK_MEM + 13 * 4).unwrap().to_le_bytes(),
        );

        assert_eq!(
            [api::TIMER_PS_256, 8, 62, 0],
            bot.load(api::CLOCK_MEM + 14 * 4).unwrap().to_le_bytes(),
        );
        assert_eq!(
            [8, 0, 62, 0],
            bot.load(api::CLOCK_MEM + 15 * 4).unwrap().to_le_bytes(),
        );
    }
}
