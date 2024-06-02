use crate::{BotId, QueuedBot};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct QueuedBots {
    bots: Vec<QueuedBot>,
}

impl QueuedBots {
    pub fn push(&mut self, bot: QueuedBot) -> Result<(), ()> {
        for other_bot in self.bots.iter_mut() {
            if other_bot.id == bot.id {
                *other_bot = bot;

                return Ok(());
            }
        }

        if self.bots.len() >= 64 {
            return Err(());
        }

        self.bots.push(bot);

        Ok(())
    }

    pub fn pop(&mut self) -> Option<QueuedBot> {
        self.bots.pop()
    }

    pub fn has(&self, id: BotId) -> bool {
        self.bots.iter().any(|bot| bot.id == id)
    }
}
