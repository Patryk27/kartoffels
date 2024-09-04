use crate::BotId;

#[derive(Debug)]
pub enum Event {
    BotSpawned { id: BotId },
    BotKilled { id: BotId },
}
