use crate::*;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BotCompass {
    dir: Option<Dir>,
    next_measurement_in: u32,
}

impl BotCompass {
    pub(super) fn tick(bot: &mut AliveBotBody) {
        if let Some(time) = bot.compass.next_measurement_in.checked_sub(1) {
            bot.compass.next_measurement_in = time;
        } else {
            bot.compass.dir = Some(bot.dir);
            bot.compass.next_measurement_in = 128_000;
        }
    }

    pub(super) fn load(bot: &mut AliveBotBody, addr: u32) -> Result<u32, ()> {
        match addr {
            api::MEM_COMPASS => Ok(match bot.compass.dir.take() {
                None => 0,
                Some(Dir::N) => 1,
                Some(Dir::E) => 2,
                Some(Dir::S) => 3,
                Some(Dir::W) => 4,
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

        assert_eq!(Ok(1), bot.load(api::MEM_COMPASS));
        assert_eq!(Ok(0), bot.load(api::MEM_COMPASS));

        // ---

        bot.dir = Dir::N;

        for _ in 0..128_000 {
            BotCompass::tick(&mut bot);
        }

        bot.dir = Dir::E;

        BotCompass::tick(&mut bot);

        assert_eq!(Ok(2), bot.load(api::MEM_COMPASS));
        assert_eq!(Ok(0), bot.load(api::MEM_COMPASS));

        // ---

        bot.dir = Dir::N;

        for _ in 0..128_000 {
            BotCompass::tick(&mut bot);
        }

        bot.dir = Dir::S;

        BotCompass::tick(&mut bot);

        assert_eq!(Ok(3), bot.load(api::MEM_COMPASS));
        assert_eq!(Ok(0), bot.load(api::MEM_COMPASS));

        // ---

        bot.dir = Dir::N;

        for _ in 0..128_000 {
            BotCompass::tick(&mut bot);
        }

        bot.dir = Dir::W;

        BotCompass::tick(&mut bot);

        assert_eq!(Ok(4), bot.load(api::MEM_COMPASS));
        assert_eq!(Ok(0), bot.load(api::MEM_COMPASS));
    }
}
