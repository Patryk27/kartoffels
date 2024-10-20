use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BotEvents {
    entries: VecDeque<Arc<BotEvent>>,
}

impl BotEvents {
    const LENGTH: usize = 128;

    pub fn add(&mut self, msg: impl Into<String>) {
        while self.entries.len() >= Self::LENGTH {
            self.entries.pop_front();
        }

        self.entries.push_back(Arc::new(BotEvent {
            at: Utc::now(),
            msg: msg.into(),
        }));
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BotEvent {
    pub at: DateTime<Utc>,
    pub msg: String,
}
