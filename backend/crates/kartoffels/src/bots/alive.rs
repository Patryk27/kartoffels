use crate::{AliveBot, BotId};
use anyhow::Result;
use glam::IVec2;
use maybe_owned::MaybeOwned;
use rand::prelude::SliceRandom;
use rand::RngCore;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{hash_map, HashMap};
use std::mem;

#[derive(Clone, Debug, Default)]
pub struct AliveBots {
    entries: HashMap<BotId, AliveBot>,
    pos_to_id: HashMap<IVec2, BotId>,
    id_to_pos: HashMap<BotId, IVec2>,
}

impl AliveBots {
    pub fn add(&mut self, id: BotId, pos: IVec2, bot: AliveBot) {
        assert!(!self.entries.contains_key(&id));
        assert!(!self.pos_to_id.contains_key(&pos));

        self.entries.insert(id, bot);
        self.pos_to_id.insert(pos, id);
        self.id_to_pos.insert(id, pos);
    }

    pub fn relocate(&mut self, id: BotId, new_pos: IVec2) {
        assert!(!self.pos_to_id.contains_key(&new_pos),);

        let hash_map::RawEntryMut::Occupied(entry) =
            self.id_to_pos.raw_entry_mut().from_key(&id)
        else {
            unreachable!();
        };

        let old_pos = mem::replace(entry.into_mut(), new_pos);
        let id = self.pos_to_id.remove(&old_pos).unwrap();

        self.pos_to_id.insert(new_pos, id);
    }

    pub fn remove(&mut self, id: BotId) -> AliveBot {
        let pos = self.id_to_pos.remove(&id).unwrap();
        let bot = self.entries.remove(&id).unwrap();

        self.pos_to_id.remove(&pos).unwrap();

        bot
    }

    pub fn lookup_by_pos(&self, pos: IVec2) -> Option<BotId> {
        self.pos_to_id.get(&pos).copied()
    }

    pub fn has(&self, id: BotId) -> bool {
        self.entries.contains_key(&id)
    }

    pub fn try_get(&self, id: BotId) -> Option<AliveBotEntry> {
        Some(AliveBotEntry {
            id,
            pos: *self.id_to_pos.get(&id)?,
            bot: self.entries.get(&id)?,
        })
    }

    pub fn try_get_mut(
        &mut self,
        id: BotId,
    ) -> Option<(AliveBotEntryMut, AliveBotsLocator)> {
        let entry = AliveBotEntryMut {
            pos: *self.id_to_pos.get(&id)?,
            bot: self.entries.get_mut(&id)?,
        };

        let locator = AliveBotsLocator {
            pos_to_id: &self.pos_to_id,
        };

        Some((entry, locator))
    }

    pub fn iter(&self) -> impl Iterator<Item = AliveBotEntry> + '_ {
        self.entries.iter().map(|(id, bot)| AliveBotEntry {
            id: *id,
            pos: self.id_to_pos[id],
            bot,
        })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn pick_ids(&self, rng: &mut impl RngCore) -> Vec<BotId> {
        let mut ids: Vec<_> = self.entries.keys().copied().collect();

        ids.shuffle(rng);
        ids
    }

    pub fn locator(&self) -> AliveBotsLocator {
        AliveBotsLocator {
            pos_to_id: &self.pos_to_id,
        }
    }
}

#[derive(Debug)]
pub struct AliveBotEntry<'a> {
    pub id: BotId,
    pub pos: IVec2,
    pub bot: &'a AliveBot,
}

#[derive(Debug)]
pub struct AliveBotEntryMut<'a> {
    pub pos: IVec2,
    pub bot: &'a mut AliveBot,
}

#[derive(Debug)]
pub struct AliveBotsLocator<'a> {
    pos_to_id: &'a HashMap<IVec2, BotId>,
}

impl AliveBotsLocator<'_> {
    pub fn contains(&self, pos: IVec2) -> bool {
        self.pos_to_id.contains_key(&pos)
    }
}

impl Serialize for AliveBots {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let proxy = SerializedAliveBots {
            bots: self
                .entries
                .iter()
                .map(|(id, bot)| SerializedAliveBot {
                    id: *id,
                    pos: self.id_to_pos[id],
                    bot: MaybeOwned::Borrowed(bot),
                })
                .collect(),
        };

        proxy.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AliveBots {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut this = Self::default();
        let proxy = SerializedAliveBots::deserialize(deserializer)?;

        for entry in proxy.bots {
            this.add(entry.id, entry.pos, entry.bot.into_owned());
        }

        Ok(this)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
struct SerializedAliveBots<'a> {
    bots: Vec<SerializedAliveBot<'a>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SerializedAliveBot<'a> {
    id: BotId,
    pos: IVec2,

    #[serde(flatten)]
    bot: MaybeOwned<'a, AliveBot>,
}
