use crate::{BotId, QueuedBot};
use ahash::AHashMap;
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
        let bot = &self.entries[place];

        Some(QueuedBotEntry { bot, place })
    }

    pub fn get_mut(&mut self, id: BotId) -> Option<&mut QueuedBot> {
        let place = *self.id_to_idx.get(&id)?;

        Some(&mut self.entries[place])
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
        serializer.collect_seq(self.entries.iter())
    }
}

impl<'de> Deserialize<'de> for QueuedBots {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut this = Self::default();
        let bots = Vec::deserialize(deserializer)?;

        for bot in bots {
            this.entries.push_back(bot);
        }

        this.index();

        Ok(this)
    }
}

#[derive(Debug)]
pub struct QueuedBotEntry<'a> {
    pub bot: &'a QueuedBot,
    pub place: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bot(id: u64) -> QueuedBot {
        QueuedBot {
            id: id.into(),
            requeued: false,
            bot: Default::default(),
        }
    }

    #[test]
    fn smoke() {
        let mut target = QueuedBots::default();

        target.push(bot(10));
        target.push(bot(20));
        target.push(bot(30));
        target.push(bot(40));
        target.push(bot(50));

        for id in [10, 20, 30, 40, 50] {
            let id = BotId::from(id);

            assert_eq!(id, target.get(id).unwrap().bot.id);
        }

        // ---

        assert_eq!(BotId::from(10), target.pop().unwrap().id);
        assert_eq!(BotId::from(20), target.pop().unwrap().id);
        assert_eq!(BotId::from(30), target.pop().unwrap().id);

        assert!(target.get(10.into()).is_none());
        assert!(target.get(30.into()).is_none());

        assert_eq!(BotId::from(40), target.get(40.into()).unwrap().bot.id);
        assert_eq!(BotId::from(50), target.get(50.into()).unwrap().bot.id);

        // ---

        assert_eq!(BotId::from(40), target.pop().unwrap().id);
        assert_eq!(BotId::from(50), target.pop().unwrap().id);
        assert!(target.pop().is_none());
    }
}
