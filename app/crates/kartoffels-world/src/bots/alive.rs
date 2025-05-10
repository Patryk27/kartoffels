use crate::{AliveBot, BotId};
use ahash::AHashMap;
use glam::IVec2;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Manages alive bots.
///
/// It's essentially `HashMap<BotId, AliveBot>`, but more optimized and with a
/// couple of helper functions (for looking up bots via their position etc.).
#[derive(Clone, Debug, Default)]
pub struct AliveBots {
    entries: Vec<Option<Box<AliveBot>>>,
    id_to_idx: AHashMap<BotId, u8>,
    pos_to_id: AHashMap<IVec2, BotId>,
    count: usize,
}

impl AliveBots {
    /// Adds given bot into the collection.
    ///
    /// This function doesn't keep track of the policy, it's on the caller's
    /// side to make sure the policy is fine with this.
    ///
    /// # Panics
    ///
    /// This function panics if adding this bot would exceed the maximum limit
    /// of `u8::MAX` alive bots. Normally this shouldn't happen, because policy
    /// doesn't allow to specify more than 255 bots anyway.
    pub fn add(&mut self, bot: Box<AliveBot>) {
        for (idx, slot) in self.entries.iter_mut().enumerate() {
            let idx = idx as u8;

            if slot.is_none() {
                self.id_to_idx.insert(bot.id, idx);
                self.pos_to_id.insert(bot.pos, bot.id);
                self.count += 1;

                *slot = Some(bot);
                return;
            }
        }

        let idx =
            u8::try_from(self.entries.len()).expect("too many alive bots");

        self.id_to_idx.insert(bot.id, idx);
        self.pos_to_id.insert(bot.pos, bot.id);
        self.entries.push(Some(bot));
        self.count += 1;
    }

    /// Returns bot with the specified id.
    ///
    /// Calling this function during [`crate::bots::tick()`] will return `None`
    /// for the currently-ticked bot, see [`Self::begin()`].
    pub fn get(&self, id: BotId) -> Option<&AliveBot> {
        let idx = *self.id_to_idx.get(&id)?;
        let bot = self.entries[idx as usize].as_ref()?;

        Some(bot)
    }

    /// Removes bot with the specified id.
    ///
    /// Calling this function during [`crate::bots::tick()`] will return `None`
    /// for the currently-ticked bot, see [`Self::begin()`].
    pub fn remove(&mut self, id: BotId) -> Option<Box<AliveBot>> {
        let idx = self.id_to_idx.remove(&id)?;
        let bot = self.entries[idx as usize].take().unwrap();

        self.pos_to_id.remove(&bot.pos).unwrap();
        self.count -= 1;

        Some(bot)
    }

    pub fn remove_at(&mut self, pos: IVec2) -> Option<Box<AliveBot>> {
        self.remove(self.lookup_at(pos)?)
    }

    pub fn lookup_at(&self, pos: IVec2) -> Option<BotId> {
        self.pos_to_id.get(&pos).copied()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut AliveBot> {
        self.entries.iter_mut().flatten().map(|bot| &mut **bot)
    }

    pub fn contains(&self, id: BotId) -> bool {
        self.id_to_idx.contains_key(&id)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn count(&self) -> usize {
        self.count
    }

    /// Removes bot with the specified index.
    ///
    /// This function is meant to be used in tandem with [`Self::commit()`] -
    /// together both functions allow to modify a bot while also retaining
    /// access to `self`.
    ///
    /// i.e. they allow to do something like `self.get_mut()` paired with
    /// `self.iter()`, which comes handy for radar and whatnot.
    pub fn begin(&mut self, idx: usize) -> Option<Box<AliveBot>> {
        self.entries[idx].take()
    }

    /// See [`Self::begin()`].
    pub fn commit(
        &mut self,
        idx: usize,
        id: BotId,
        pos: IVec2,
        bot: Option<Box<AliveBot>>,
    ) {
        if let Some(bot) = bot {
            if bot.pos != pos {
                self.pos_to_id.remove(&pos).unwrap();
                self.pos_to_id.insert(bot.pos, bot.id);
            }

            self.entries[idx] = Some(bot);
        } else {
            self.id_to_idx.remove(&id);
            self.pos_to_id.remove(&pos);
            self.count -= 1;
        }
    }
}

impl Serialize for AliveBots {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self.entries.iter().flatten())
    }
}

impl<'de> Deserialize<'de> for AliveBots {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut this = Self::default();
        let bots = Vec::deserialize(deserializer)?;

        for bot in bots {
            this.add(bot);
        }

        Ok(this)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AliveBotBody;
    use glam::ivec2;

    fn bot(id: u64, pos: IVec2) -> Box<AliveBot> {
        Box::new(AliveBot {
            body: AliveBotBody {
                id: BotId::new(id),
                pos,
                ..Default::default()
            },
            ..Default::default()
        })
    }

    #[test]
    fn smoke() {
        let mut target = AliveBots::default();

        target.add(bot(1, ivec2(10, 10)));
        target.add(bot(2, ivec2(20, 20)));
        target.add(bot(3, ivec2(30, 30)));
        target.add(bot(4, ivec2(40, 40)));
        target.add(bot(5, ivec2(50, 50)));

        assert_eq!(5, target.len());
        assert_eq!(5, target.count());

        assert_eq!(Some(BotId::new(1)), target.lookup_at(ivec2(10, 10)));
        assert_eq!(Some(BotId::new(2)), target.lookup_at(ivec2(20, 20)));
        assert_eq!(Some(BotId::new(3)), target.lookup_at(ivec2(30, 30)));
        assert_eq!(Some(BotId::new(4)), target.lookup_at(ivec2(40, 40)));
        assert_eq!(Some(BotId::new(5)), target.lookup_at(ivec2(50, 50)));
        assert_eq!(None, target.lookup_at(ivec2(10, 20)));

        assert!(target.contains(BotId::new(1)));
        assert!(target.contains(BotId::new(2)));
        assert!(target.contains(BotId::new(3)));
        assert!(target.contains(BotId::new(4)));
        assert!(target.contains(BotId::new(5)));
        assert!(!target.contains(BotId::new(6)));

        // ---

        assert_eq!(BotId::new(4), target.remove(BotId::new(4)).unwrap().id);
        assert!(target.remove(BotId::new(4)).is_none());

        assert_eq!(5, target.len());
        assert_eq!(4, target.count());

        assert_eq!(Some(BotId::new(1)), target.lookup_at(ivec2(10, 10)));
        assert_eq!(Some(BotId::new(2)), target.lookup_at(ivec2(20, 20)));
        assert_eq!(Some(BotId::new(3)), target.lookup_at(ivec2(30, 30)));
        assert_eq!(None, target.lookup_at(ivec2(40, 40)));
        assert_eq!(Some(BotId::new(5)), target.lookup_at(ivec2(50, 50)));

        assert!(target.contains(BotId::new(1)));
        assert!(target.contains(BotId::new(2)));
        assert!(target.contains(BotId::new(3)));
        assert!(!target.contains(BotId::new(4)));
        assert!(target.contains(BotId::new(5)));
    }
}
