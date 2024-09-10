use crate::BotId;

#[derive(Debug)]
pub enum Event {
    BotCreated { id: BotId },
    BotSpawned { id: BotId },
    BotKilled { id: BotId },
}
