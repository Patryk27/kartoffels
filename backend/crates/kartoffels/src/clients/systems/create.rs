use crate::{
    BotEntryMut, Client, ClientBot, ClientBotEvents, CreateClient, World,
};

pub fn run(world: &mut World) {
    while let Some(CreateClient { id, tx }) = world.events.recv() {
        let bot = id.map(|id| {
            let events = world
                .bots
                .get_mut(id)
                .map(|bot| match bot {
                    BotEntryMut::Queued(bot) => &mut bot.bot.events,
                    BotEntryMut::Alive(bot) => &mut bot.events,
                    BotEntryMut::Dead(bot) => &mut bot.events,
                })
                .map(|events| ClientBotEvents {
                    rx: events.subscribe(),
                    init: events.iter().cloned().collect(),
                });

            ClientBot { id, events }
        });

        world.clients.push(Client {
            tx,
            bot,
            is_fresh: true,
        });
    }
}
