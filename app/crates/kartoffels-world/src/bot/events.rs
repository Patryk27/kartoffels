use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BotEvents {
    entries: VecDeque<Arc<BotEvent>>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    snapshot: Option<Arc<VecDeque<Arc<BotEvent>>>>,
}

impl BotEvents {
    const LENGTH: usize = 128;

    pub fn add(&mut self, msg: impl Into<String>) {
        while self.entries.len() >= Self::LENGTH {
            self.entries.pop_back();
        }

        self.entries.push_front(Arc::new(BotEvent {
            at: Utc::now(),
            msg: msg.into(),
        }));

        self.snapshot = None;
    }

    pub fn snapshot(&mut self) -> Arc<VecDeque<Arc<BotEvent>>> {
        self.snapshot
            .get_or_insert_with(|| Arc::new(self.entries.clone()))
            .clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BotEvent {
    pub at: DateTime<Utc>,
    pub msg: String,
}
