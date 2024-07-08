use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::broadcast;

pub type BotEventTx = broadcast::Sender<Arc<BotEvent>>;
pub type BotEventRx = broadcast::Receiver<Arc<BotEvent>>;

#[derive(Clone, Debug)]
pub struct BotEvents {
    entries: VecDeque<Arc<BotEvent>>,
    changes: BotEventTx,
}

impl BotEvents {
    const LENGTH: usize = 128;

    pub fn add(&mut self, msg: String) {
        while self.entries.len() >= Self::LENGTH {
            self.entries.pop_front();
        }

        let event = Arc::new(BotEvent {
            at: Utc::now(),
            msg,
        });

        self.entries.push_back(event.clone());
        _ = self.changes.send(event);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arc<BotEvent>> {
        self.entries.iter()
    }

    pub fn subscribe(&self) -> BotEventRx {
        self.changes.subscribe()
    }
}

impl Default for BotEvents {
    fn default() -> Self {
        Self {
            entries: Default::default(),
            changes: BotEventTx::new(Self::LENGTH),
        }
    }
}

impl Serialize for BotEvents {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self.entries.iter())
    }
}

impl<'de> Deserialize<'de> for BotEvents {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut this = Self::default();
        let entries = Vec::deserialize(deserializer)?;

        for entry in entries {
            this.entries.push_back(entry);
        }

        Ok(this)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BotEvent {
    pub at: DateTime<Utc>,
    pub msg: String,
}
