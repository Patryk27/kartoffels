use crate::{BotId, QueuedBot};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct QueuedBots {
    bots: VecDeque<QueuedBot>,
}

impl QueuedBots {
    pub fn push(&mut self, bot: QueuedBot) -> Result<(), ()> {
        for other_bot in self.bots.iter_mut() {
            if other_bot.id == bot.id {
                *other_bot = bot;

                return Ok(());
            }
        }

        // TODO make configurable
        if self.bots.len() >= 64 {
            return Err(());
        }

        self.bots.push_back(bot);

        Ok(())
    }

    pub fn pop(&mut self) -> Option<QueuedBot> {
        self.bots.pop_front()
    }

    pub fn has(&self, id: BotId) -> bool {
        self.bots.iter().any(|bot| bot.id == id)
    }

    pub fn len(&self) -> usize {
        self.bots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bots.is_empty()
    }
}
