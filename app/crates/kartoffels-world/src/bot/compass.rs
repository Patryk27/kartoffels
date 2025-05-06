use crate::*;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BotCompass {
    dir: Option<Dir>,
    cooldown: u32,
}

impl BotCompass {
    pub(super) fn tick(bot: &mut AliveBotBody) {
        if let Some(time) = bot.compass.cooldown.checked_sub(1) {
            bot.compass.cooldown = time;
        } else {
            bot.compass.dir = Some(bot.dir);
            bot.compass.cooldown = 128_000;

            bot.irq
                .raise(api::IRQ_COMPASS_READY, [1 + bot.dir as u8, 0x00, 0x00]);
        }
    }

    pub(super) fn load(bot: &mut AliveBotBody, addr: u32) -> Result<u32, ()> {
        match addr {
            api::COMPASS_MEM => Ok(match bot.compass.dir.take() {
                None => 0,
                Some(dir) => 1 + dir as u32,
            }),

            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let mut bot = AliveBot::default();

        BotCompass::tick(&mut bot);

        assert_eq!(Ok(1), bot.load(api::COMPASS_MEM));
        assert_eq!(Ok(0), bot.load(api::COMPASS_MEM));

        assert_eq!(
            Some([api::IRQ_COMPASS_READY, 0x01, 0x00, 0x00]),
            bot.irq.take_le(),
        );

        for (dir, idx) in [(Dir::N, 1), (Dir::E, 2), (Dir::S, 3), (Dir::W, 4)] {
            bot.dir = dir.turned_back();

            for _ in 0..128_000 {
                BotCompass::tick(&mut bot);

                assert_eq!(None, bot.irq.take());
            }

            bot.dir = dir;

            BotCompass::tick(&mut bot);

            assert_eq!(Ok(idx), bot.load(api::COMPASS_MEM));
            assert_eq!(Ok(0), bot.load(api::COMPASS_MEM));

            assert_eq!(
                Some([api::IRQ_COMPASS_READY, idx as u8, 0x00, 0x00]),
                bot.irq.take_le(),
            );
        }
    }
}
