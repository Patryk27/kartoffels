use crate::{BotId, DeadBot};
use ahash::AHashSet;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::VecDeque;
use tracing::debug;

#[derive(Clone, Debug, Default)]
pub struct DeadBots {
    entries: VecDeque<DeadBot>,
    index: AHashSet<BotId>,
}

impl DeadBots {
    const MAX_ENTRIES: usize = 4 * 1024;

    pub fn add(&mut self, bot: DeadBot) {
        if self.entries.len() >= Self::MAX_ENTRIES {
            // Unwrap-safety: We've just checked that `self.entries` is not
            // empty
            let entry = self.entries.pop_front().unwrap();

            debug!(id=?entry.id, "forgetting bot");

            self.index.remove(&entry.id);
        }

        self.index.insert(bot.id);
        self.entries.push_back(bot);
    }

    pub fn remove(&mut self, id: BotId) {
        if self.index.remove(&id) {
            // Calling `.retain()` is suboptimal, but `self.remove()` is not a
            // particularly frequently called function, so bothering with a
            // separate index is not worth it here
            self.entries.retain(|bot| bot.id != id);
        }
    }

    pub fn contains(&self, id: BotId) -> bool {
        self.index.contains(&id)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut DeadBot> + '_ {
        self.entries.iter_mut()
    }
}

impl Serialize for DeadBots {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self.entries.iter())
    }
}

impl<'de> Deserialize<'de> for DeadBots {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut this = Self::default();
        let bots = Vec::<DeadBot>::deserialize(deserializer)?;

        for bot in bots {
            this.index.insert(bot.id);
            this.entries.push_back(bot);
        }

        Ok(this)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bot(id: u64) -> DeadBot {
        DeadBot {
            events: Default::default(),
            id: BotId::new(id),
            serial: Default::default(),
        }
    }

    #[test]
    fn smoke() {
        let mut target = DeadBots::default();

        for id in 1..10 {
            target.add(bot(id as u64));
        }

        for id in 1..10 {
            assert!(target.contains(BotId::new(id as u64)));
        }

        assert!(!target.contains(BotId::new(10)));

        // ---

        target.remove(BotId::new(3));
        target.remove(BotId::new(5));

        assert!(target.contains(BotId::new(1)));
        assert!(target.contains(BotId::new(2)));
        assert!(!target.contains(BotId::new(3)));
        assert!(target.contains(BotId::new(4)));
        assert!(!target.contains(BotId::new(5)));
        assert!(target.contains(BotId::new(6)));
        assert!(target.contains(BotId::new(7)));
        assert!(target.contains(BotId::new(8)));
        assert!(target.contains(BotId::new(9)));

        // ---

        let expected = vec![
            BotId::new(1),
            BotId::new(2),
            BotId::new(4),
            BotId::new(6),
            BotId::new(7),
            BotId::new(8),
            BotId::new(9),
        ];

        let actual: Vec<_> = target.iter_mut().map(|bot| bot.id).collect();

        assert_eq!(expected, actual);

        // ---

        for id in 0..(DeadBots::MAX_ENTRIES - 3) {
            target.add(bot(10 + id as u64));
        }

        for id in 1..=6 {
            assert!(!target.contains(BotId::new(id)));
        }

        for id in 7..=(DeadBots::MAX_ENTRIES - 2) {
            assert!(target.contains(BotId::new(id as u64)));
        }
    }
}
