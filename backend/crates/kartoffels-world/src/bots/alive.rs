use crate::{AliveBot, BotId};
use ahash::AHashMap;
use anyhow::Result;
use glam::IVec2;
use kartoffels_utils::DummyHasher;
use maybe_owned::MaybeOwned;
use rand::prelude::SliceRandom;
use rand::RngCore;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::mem;

#[derive(Clone, Debug, Default)]
pub struct AliveBots {
    entries: HashMap<BotId, AliveBot, DummyHasher>,
    pos_to_id: AHashMap<IVec2, BotId>,
    id_to_pos: HashMap<BotId, IVec2, DummyHasher>,
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
        assert!(!self.pos_to_id.contains_key(&new_pos));

        let old_pos =
            mem::replace(self.id_to_pos.get_mut(&id).unwrap(), new_pos);

        let id = self.pos_to_id.remove(&old_pos).unwrap();

        self.pos_to_id.insert(new_pos, id);
    }

    pub fn remove(&mut self, id: BotId) -> AliveBot {
        let pos = self.id_to_pos.remove(&id).unwrap();
        let bot = self.entries.remove(&id).unwrap();

        self.pos_to_id.remove(&pos).unwrap();

        bot
    }

    pub fn get_mut(&mut self, id: BotId) -> Option<AliveBotEntryMut> {
        Some(AliveBotEntryMut {
            bot: self.entries.get_mut(&id)?,
            pos: self.id_to_pos[&id],
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
    pub locator: AliveBotsLocator<'a>,
}

#[derive(Debug)]
pub struct AliveBotsLocator<'a> {
    pos_to_id: &'a AHashMap<IVec2, BotId>,
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
        serializer.collect_seq(self.entries.iter().map(|(id, bot)| {
            SerializedAliveBot {
                id: *id,
                pos: self.id_to_pos[id],
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
            this.add(bot.id, bot.pos, bot.bot.into_owned());
        }

        Ok(this)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SerializedAliveBot<'a> {
    id: BotId,
    pos: IVec2,

    #[serde(flatten)]
    bot: MaybeOwned<'a, AliveBot>,
}
