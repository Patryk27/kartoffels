use crate::{
    BotEntryMut, Conn, ConnBot, ConnBotEvents, CreateConnection, World,
};

pub fn run(world: &mut World) {
    while let Some(CreateConnection { id, tx }) = world.events.recv() {
        let bot = id.map(|id| {
            let events = world
                .bots
                .get_mut(id)
                .map(|bot| match bot {
                    BotEntryMut::Queued(bot) => &mut bot.bot.events,
                    BotEntryMut::Alive(bot) => &mut bot.events,
                    BotEntryMut::Dead(bot) => &mut bot.events,
                })
                .map(|events| ConnBotEvents {
                    rx: events.subscribe(),
                    init: events.iter().cloned().collect(),
                });

            ConnBot { id, events }
        });

        world.conns.push(Conn {
            tx,
            bot,
            is_fresh: true,
        });
    }
}
