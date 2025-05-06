use crate::*;

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

    pub fn add(&mut self, clock: &Clock, msg: impl Into<String>) {
        if self.entries.len() >= Self::LENGTH {
            self.entries.pop_back();
        }

        self.entries.push_front(Arc::new(BotEvent {
            at: clock.now(),
            msg: msg.into(),
        }));

        self.snapshot = None;
    }

    #[cfg(test)]
    pub fn newest(&self) -> Option<&str> {
        self.entries.front().map(|event| event.msg.as_str())
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

impl BotEvent {
    pub fn test(msg: impl Into<String>) -> Self {
        Self {
            at: "2018-01-01T12:00:00Z".parse().unwrap(),
            msg: msg.into(),
        }
    }
}
