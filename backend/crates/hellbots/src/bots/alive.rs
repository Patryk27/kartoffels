use crate::{AliveBot, BotId, BotSnapshot};
use anyhow::{Context, Result};
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
    #[allow(clippy::result_large_err)]
    pub fn add(
        &mut self,
        id: BotId,
        pos: IVec2,
        bot: AliveBot,
    ) -> Result<(), AliveBot> {
        if self.entries.len() >= 64 {
            // TODO make the max length configurable
            return Err(bot);
        }

        self.entries.insert(id, bot);
        self.pos_to_id.insert(pos, id);
        self.id_to_pos.insert(id, pos);

        Ok(())
    }

    pub fn relocate(&mut self, id: BotId, new_pos: IVec2) {
        if self.pos_to_id.contains_key(&new_pos) {
            return;
        }

        let hash_map::RawEntryMut::Occupied(entry) =
            self.id_to_pos.raw_entry_mut().from_key(&id)
        else {
            unreachable!();
        };

        let old_pos = mem::replace(entry.into_mut(), new_pos);
        let id = self.pos_to_id.remove(&old_pos).unwrap();

        self.pos_to_id.insert(new_pos, id);
    }

    pub fn remove(&mut self, id: BotId) -> Result<AliveBot> {
        let pos = self
            .id_to_pos
            .remove(&id)
            .with_context(|| format!("couldn't find bot #{}", id))?;

        let bot = self.entries.remove(&id).unwrap();

        self.pos_to_id.remove(&pos);

        Ok(bot)
    }

    pub fn id_to_bot_err(&self, id: BotId) -> Result<&AliveBot> {
        self.entries
            .get(&id)
            .with_context(|| format!("couldn't find bot {id}"))
            .context("id_to_bot() failed")
    }

    pub fn id_to_pos_err(&self, id: BotId) -> Result<IVec2> {
        self.id_to_pos
            .get(&id)
            .copied()
            .with_context(|| format!("couldn't find bot {id}"))
            .context("id_to_pos() failed")
    }

    pub fn pos_to_id(&self, pos: IVec2) -> Option<BotId> {
        self.pos_to_id.get(&pos).copied()
    }

    pub fn entry_mut(
        &mut self,
        id: BotId,
    ) -> Option<(IVec2, &mut AliveBot, AliveBotsLocator)> {
        let pos = *self.id_to_pos.get(&id)?;
        let bot = self.entries.get_mut(&id).unwrap();

        let bots = AliveBotsLocator {
            pos_to_id: &self.pos_to_id,
        };

        Some((pos, bot, bots))
    }

    pub fn has(&self, id: BotId) -> bool {
        self.entries.contains_key(&id)
    }

    pub fn locator(&self) -> AliveBotsLocator {
        AliveBotsLocator {
            pos_to_id: &self.pos_to_id,
        }
    }

    pub fn ids(&self) -> impl Iterator<Item = BotId> + '_ {
        self.entries.keys().copied()
    }

    pub fn pick_ids(&self, rng: &mut impl RngCore) -> Vec<BotId> {
        let mut ids: Vec<_> = self.ids().collect();

        ids.shuffle(rng);
        ids
    }

    pub fn snapshots(&self) -> impl Iterator<Item = (BotId, BotSnapshot)> + '_ {
        self.entries.iter().map(|(id, bot)| {
            let snapshot = BotSnapshot {
                pos: self.id_to_pos[id],
                uart: bot.uart.to_string(),
            };

            (*id, snapshot)
        })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
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
            this.add(entry.id, entry.pos, entry.bot.into_owned())
                .unwrap(); // TODO
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
