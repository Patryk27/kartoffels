use crate::{BotId, QueuedBot};
use ahash::AHashMap;
use maybe_owned::MaybeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::VecDeque;

#[derive(Clone, Debug, Default)]
pub struct QueuedBots {
    entries: VecDeque<QueuedBot>,
    id_to_idx: AHashMap<BotId, usize>,
}

impl QueuedBots {
    pub fn push(&mut self, bot: QueuedBot) {
        self.entries.push_back(bot);
        self.index();
    }

    pub fn pop(&mut self) -> Option<QueuedBot> {
        if let Some(entry) = self.entries.pop_front() {
            self.index();

            Some(entry)
        } else {
            None
        }
    }

    pub fn get(&self, id: BotId) -> Option<QueuedBotEntry> {
        let place = *self.id_to_idx.get(&id)?;

        Some(QueuedBotEntry {
            place,
            requeued: self.entries[place].requeued,
        })
    }

    pub fn has(&self, id: BotId) -> bool {
        self.id_to_idx.contains_key(&id)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn index(&mut self) {
        self.id_to_idx.clear();

        self.id_to_idx.extend(
            self.entries
                .iter()
                .enumerate()
                .map(|(idx, bot)| (bot.id, idx)),
        );
    }
}

impl Serialize for QueuedBots {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let proxy = SerializedQueuedBots {
            bots: self.entries.iter().map(MaybeOwned::Borrowed).collect(),
        };

        proxy.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for QueuedBots {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut this = Self::default();
        let proxy = SerializedQueuedBots::deserialize(deserializer)?;

        for entry in proxy.bots {
            this.entries.push_back(entry.into_owned());
        }

        this.index();

        Ok(this)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QueuedBotEntry {
    pub place: usize,
    pub requeued: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
struct SerializedQueuedBots<'a> {
    bots: Vec<MaybeOwned<'a, QueuedBot>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn qbot(id: u64) -> QueuedBot {
        QueuedBot {
            id: id.into(),
            requeued: false,
            bot: Default::default(),
        }
    }

    #[test]
    fn smoke() {
        let mut target = QueuedBots::default();

        target.push(qbot(10));
        target.push(qbot(20));
        target.push(qbot(30));
        target.push(qbot(40));
        target.push(qbot(50));

        let entry = |place: usize| QueuedBotEntry {
            place,
            requeued: false,
        };

        assert_eq!(Some(entry(0)), target.get(10.into()));
        assert_eq!(Some(entry(1)), target.get(20.into()));
        assert_eq!(Some(entry(2)), target.get(30.into()));
        assert_eq!(Some(entry(3)), target.get(40.into()));
        assert_eq!(Some(entry(4)), target.get(50.into()));

        // ---

        assert_eq!(BotId::from(10), target.pop().unwrap().id);
        assert_eq!(BotId::from(20), target.pop().unwrap().id);
        assert_eq!(BotId::from(30), target.pop().unwrap().id);

        assert_eq!(None, target.get(10.into()));
        assert_eq!(None, target.get(30.into()));

        assert_eq!(Some(entry(0)), target.get(40.into()));
        assert_eq!(Some(entry(1)), target.get(50.into()));

        // ---

        assert_eq!(BotId::from(40), target.pop().unwrap().id);
        assert_eq!(BotId::from(50), target.pop().unwrap().id);
        assert!(target.pop().is_none());
    }
}
