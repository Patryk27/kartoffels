use crate::{AliveBotBody, World};

pub fn tick(world: &mut World) {
    let mut idx = 0;
    let len = world.bots.alive.len();

    while idx < len {
        let Some(mut bot) = world.bots.alive.begin(idx) else {
            idx += 1;
            continue;
        };

        let id = bot.id;
        let pos = bot.pos;
        let mut ticks = world.clock.ticks();

        let bot = loop {
            if ticks == 0 {
                break Some(bot);
            }

            match bot.tick(world) {
                Ok(()) => {
                    if bot.pos == AliveBotBody::FELL_INTO_VOID {
                        world.kill_bot(bot, "fell into the void".into(), None);
                        break None;
                    } else {
                        ticks -= 1;
                        continue;
                    }
                }

                Err(err) => {
                    world.kill_bot(
                        bot,
                        format!("firmware crashed: {err}"),
                        None,
                    );
                    break None;
                }
            };
        };

        world.bots.alive.commit(idx, id, pos, bot);
        idx += 1;
    }
}
