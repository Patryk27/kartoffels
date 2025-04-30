use crate::*;

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

    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    pub(crate) fn tick(bot: &mut AliveBotBody) {
        bot.timer.ticks += 1;
    }

    pub(crate) fn load(bot: &AliveBotBody, addr: u32) -> Result<u32, ()> {
        match addr {
            api::MEM_TIMER => Ok(bot.timer.seed),
            addr if addr == api::MEM_TIMER + 4 => Ok(bot.timer.ticks as u32),

            _ => Err(()),
        }
    }

    pub(crate) fn store(
        _bot: &mut AliveBotBody,
        _addr: u32,
        _val: u32,
    ) -> Result<(), ()> {
        Err(())
    }
}
