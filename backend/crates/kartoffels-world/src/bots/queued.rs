use crate::{BotId, QueuedBot};
use ahash::AHashMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::VecDeque;

#[derive(Clone, Debug, Default)]
pub struct QueuedBots {
    entries: VecDeque<QueuedBot>,
    id_to_place: AHashMap<BotId, u8>,
}

impl QueuedBots {
    pub fn push(&mut self, bot: QueuedBot) {
        self.entries.push_back(bot);
        self.reindex();
    }

    pub fn pop(&mut self) -> Option<QueuedBot> {
        let entry = self.entries.pop_front()?;

        self.reindex();

        Some(entry)
    }

    pub fn peek(&self) -> Option<&QueuedBot> {
        self.entries.front()
    }

    pub fn remove(&mut self, id: BotId) {
        let Some(place) = self.id_to_place.remove(&id) else {
            return;
        };

        self.entries.remove(place as usize);
        self.reindex();
    }

    #[cfg(test)]
    pub fn get(&self, id: BotId) -> Option<QueuedBotEntry> {
        let place = *self.id_to_place.get(&id)?;
        let bot = &self.entries[place as usize];

        Some(QueuedBotEntry { id, bot, place })
    }

    pub fn contains(&self, id: BotId) -> bool {
        self.id_to_place.contains_key(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = QueuedBotEntry> {
        self.id_to_place.iter().map(|(&id, &place)| QueuedBotEntry {
            id,
            bot: &self.entries[place as usize],
            place,
        })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    fn reindex(&mut self) {
        self.id_to_place.clear();

        self.id_to_place.extend(
            self.entries
                .iter()
                .enumerate()
                .map(|(place, bot)| (bot.id, place as u8)),
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

        this.reindex();

        Ok(this)
    }
}

#[derive(Debug)]
pub struct QueuedBotEntry<'a> {
    pub id: BotId,
    pub bot: &'a QueuedBot,
    pub place: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bot(id: u64) -> QueuedBot {
        QueuedBot {
            dir: None,
            events: Default::default(),
            fw: Default::default(),
            id: BotId::new(id),
            oneshot: false,
            pos: None,
            requeued: false,
            serial: Default::default(),
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
            let id = BotId::new(id);

            assert_eq!(id, target.get(id).unwrap().bot.id);
        }

        // ---

        assert_eq!(BotId::new(10), target.pop().unwrap().id);
        assert_eq!(BotId::new(20), target.pop().unwrap().id);
        assert_eq!(BotId::new(30), target.pop().unwrap().id);

        assert!(target.get(BotId::new(10)).is_none());
        assert!(target.get(BotId::new(30)).is_none());

        assert_eq!(BotId::new(40), target.get(BotId::new(40)).unwrap().bot.id);
        assert_eq!(BotId::new(50), target.get(BotId::new(50)).unwrap().bot.id);

        // ---

        assert_eq!(BotId::new(40), target.pop().unwrap().id);
        assert_eq!(BotId::new(50), target.pop().unwrap().id);
        assert!(target.pop().is_none());
    }
}
