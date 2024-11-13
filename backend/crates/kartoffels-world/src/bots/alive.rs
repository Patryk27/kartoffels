use crate::{AliveBot, BotId};
use ahash::AHashMap;
use anyhow::Result;
use glam::IVec2;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, Default)]
pub struct AliveBots {
    entries: Vec<Option<Box<AliveBot>>>,
    id_to_idx: AHashMap<BotId, u8>,
    pos_to_id: AHashMap<IVec2, BotId>,
}

impl AliveBots {
    pub fn add(&mut self, bot: AliveBot) {
        for (idx, slot) in self.entries.iter_mut().enumerate() {
            let idx = idx as u8;

            if slot.is_none() {
                self.id_to_idx.insert(bot.id, idx);
                self.pos_to_id.insert(bot.pos, bot.id);

                *slot = Some(Box::new(bot));
                return;
            }
        }

        let idx =
            u8::try_from(self.entries.len()).expect("too many alive robots");

        self.id_to_idx.insert(bot.id, idx);
        self.pos_to_id.insert(bot.pos, bot.id);
        self.entries.push(Some(Box::new(bot)));
    }

    pub fn remove(&mut self, id: BotId) -> Option<AliveBot> {
        let idx = *self.id_to_idx.get(&id)?;
        let bot = self.entries[idx as usize].take().unwrap();

        self.pos_to_id.remove(&bot.pos).unwrap();

        Some(*bot)
    }

    pub fn get_by_pos(&self, pos: IVec2) -> Option<BotId> {
        self.pos_to_id.get(&pos).copied()
    }

    pub fn take(&mut self, idx: usize) -> Option<Box<AliveBot>> {
        self.entries[idx].take()
    }

    pub fn insert(
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
        }
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
        self.entries.iter().flatten().count()
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
        let bots = Vec::<AliveBot>::deserialize(deserializer)?;

        for bot in bots {
            this.add(bot);
        }

        Ok(this)
    }
}
