use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BotEvents {
    entries: VecDeque<BotEvent>,
}

impl BotEvents {
    const LENGTH: usize = 128;

    pub fn add(&mut self, msg: String) {
        while self.entries.len() >= Self::LENGTH {
            self.entries.pop_front();
        }

        self.entries.push_back(BotEvent {
            at: Utc::now(),
            msg,
        });
    }

    pub fn iter(&self) -> impl Iterator<Item = &BotEvent> {
        self.entries.iter()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BotEvent {
    pub at: DateTime<Utc>,
    pub msg: String,
}
