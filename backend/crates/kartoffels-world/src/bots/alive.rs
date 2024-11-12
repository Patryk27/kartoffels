use crate::{AliveBot, BotId};
use ahash::AHashMap;
use anyhow::Result;
use glam::IVec2;
use kartoffels_utils::DummyHasher;
use maybe_owned::MaybeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct AliveBots {
    entries: HashMap<BotId, AliveBot, DummyHasher>,
    pos_to_id: AHashMap<IVec2, BotId>,
}

impl AliveBots {
    pub fn add(&mut self, id: BotId, bot: AliveBot) {
        let was_pos_free = self.pos_to_id.insert(bot.pos, id).is_none();
        let was_entry_free = self.entries.insert(id, bot).is_none();

        assert!(was_pos_free);
        assert!(was_entry_free);
    }

    pub fn relocate(&mut self, id: BotId, new_pos: IVec2) {
        let bot = self.entries.get_mut(&id).unwrap();
        let id = self.pos_to_id.remove(&bot.pos).unwrap();

        self.pos_to_id.insert(new_pos, id);

        bot.pos = new_pos;
    }

    pub fn remove(&mut self, id: BotId) -> Option<AliveBot> {
        let bot = self.entries.remove(&id)?;

        self.pos_to_id.remove(&bot.pos).unwrap();

        Some(bot)
    }

    pub fn get_mut(&mut self, id: BotId) -> Option<AliveBotEntryMut> {
        Some(AliveBotEntryMut {
            bot: self.entries.get_mut(&id)?,
            locator: AliveBotsLocator {
                pos_to_id: &self.pos_to_id,
            },
        })
    }

    pub fn get_by_pos(&self, pos: IVec2) -> Option<BotId> {
        self.pos_to_id.get(&pos).copied()
    }

    pub fn contains(&self, id: BotId) -> bool {
        self.entries.contains_key(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = AliveBotEntry> + '_ {
        self.entries
            .iter()
            .map(|(id, bot)| AliveBotEntry { id: *id, bot })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn ids(&self) -> Vec<BotId> {
        self.entries.keys().copied().collect()
    }

    #[cfg(test)]
    pub fn locator(&self) -> AliveBotsLocator {
        AliveBotsLocator {
            pos_to_id: &self.pos_to_id,
        }
    }
}

#[derive(Debug)]
pub struct AliveBotEntry<'a> {
    pub id: BotId,
    pub bot: &'a AliveBot,
}

#[derive(Debug)]
pub struct AliveBotEntryMut<'a> {
    pub bot: &'a mut AliveBot,
    pub locator: AliveBotsLocator<'a>,
}

#[derive(Debug)]
pub struct AliveBotsLocator<'a> {
    pos_to_id: &'a AHashMap<IVec2, BotId>,
}

impl AliveBotsLocator<'_> {
    pub fn at(&self, pos: IVec2) -> Option<BotId> {
        self.pos_to_id.get(&pos).copied()
    }
}

impl Serialize for AliveBots {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self.entries.iter().map(|(id, bot)| {
            SerializedAliveBot {
                id: *id,
                bot: MaybeOwned::Borrowed(bot),
            }
        }))
    }
}

impl<'de> Deserialize<'de> for AliveBots {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut this = Self::default();
        let bots = Vec::<SerializedAliveBot>::deserialize(deserializer)?;

        for bot in bots {
            this.add(bot.id, bot.bot.into_owned());
        }

        Ok(this)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SerializedAliveBot<'a> {
    id: BotId,

    #[serde(flatten)]
    bot: MaybeOwned<'a, AliveBot>,
}
