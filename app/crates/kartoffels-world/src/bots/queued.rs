use crate::*;

#[derive(Clone, Debug, Default)]
pub struct QueuedBots {
    entries: VecDeque<Box<QueuedBot>>,
    index: AHashMap<BotId, u8>,
}

impl QueuedBots {
    pub fn push_front(&mut self, bot: Box<QueuedBot>) {
        self.entries.push_front(bot);
        self.reindex();
    }

    pub fn push_back(&mut self, bot: Box<QueuedBot>) {
        self.entries.push_back(bot);
        self.reindex();
    }

    pub fn pop_front(&mut self) -> Option<Box<QueuedBot>> {
        let entry = self.entries.pop_front()?;

        self.reindex();

        Some(entry)
    }

    #[cfg(test)]
    pub fn front(&self) -> Option<&QueuedBot> {
        self.entries.front().map(|bot| &**bot)
    }

    pub fn remove(&mut self, id: BotId) {
        let Some(idx) = self.index.remove(&id) else {
            return;
        };

        self.entries.remove(idx as usize);
        self.reindex();
    }

    pub fn contains(&self, id: BotId) -> bool {
        self.index.contains_key(&id)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = QueuedBotEntryMut> {
        self.entries.iter_mut().enumerate().map(|(idx, bot)| {
            QueuedBotEntryMut {
                bot,
                place: (idx + 1) as u8,
            }
        })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    fn reindex(&mut self) {
        self.index.clear();

        self.index.extend(
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
pub struct QueuedBotEntryMut<'a> {
    pub bot: &'a mut QueuedBot,
    pub place: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bot(id: u64) -> Box<QueuedBot> {
        Box::new(QueuedBot {
            dir: None,
            events: Default::default(),
            fw: Default::default(),
            id: BotId::new(id),
            oneshot: false,
            pos: None,
            requeued: false,
            serial: Default::default(),
        })
    }

    #[test]
    fn smoke() {
        let mut target = QueuedBots::default();

        target.push_back(bot(1));
        target.push_back(bot(2));
        target.push_back(bot(3));
        target.push_back(bot(4));
        target.push_back(bot(5));

        assert_eq!(5, target.len());
        assert!(target.contains(BotId::new(1)));
        assert!(target.contains(BotId::new(2)));
        assert!(target.contains(BotId::new(3)));
        assert!(target.contains(BotId::new(4)));
        assert!(target.contains(BotId::new(5)));
        assert!(!target.contains(BotId::new(6)));

        // ---

        let expected = vec![
            BotId::new(1),
            BotId::new(2),
            BotId::new(3),
            BotId::new(4),
            BotId::new(5),
        ];

        let actual: Vec<_> =
            target.iter_mut().map(|entry| entry.bot.id).collect();

        assert_eq!(expected, actual);

        // ---

        assert_eq!(BotId::new(1), target.front().unwrap().id);
        assert_eq!(BotId::new(1), target.pop_front().unwrap().id);
        assert!(!target.contains(BotId::new(1)));

        assert_eq!(BotId::new(2), target.front().unwrap().id);
        assert_eq!(BotId::new(2), target.pop_front().unwrap().id);
        assert!(!target.contains(BotId::new(2)));

        assert_eq!(BotId::new(3), target.front().unwrap().id);
        assert_eq!(BotId::new(3), target.pop_front().unwrap().id);
        assert!(!target.contains(BotId::new(3)));

        assert!(target.contains(BotId::new(4)));
        assert!(target.contains(BotId::new(5)));

        // ---

        assert_eq!(BotId::new(4), target.pop_front().unwrap().id);
        assert!(!target.contains(BotId::new(4)));

        assert_eq!(BotId::new(5), target.pop_front().unwrap().id);
        assert!(!target.contains(BotId::new(5)));

        assert!(target.pop_front().is_none());
        assert_eq!(0, target.len());
    }
}
