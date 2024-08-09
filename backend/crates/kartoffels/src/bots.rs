mod alive;
mod dead;
mod queued;
mod systems;

pub use self::alive::*;
pub use self::dead::*;
pub use self::queued::*;
pub use self::systems::*;
use crate::{AliveBot, BotId, DeadBot, QueuedBot};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Bots {
    pub alive: AliveBots,
    pub dead: DeadBots,
    pub queued: QueuedBots,
}

impl Bots {
    pub fn get(&self, id: BotId) -> Option<BotEntry> {
        if let Some(entry) = self.alive.get(id) {
            return Some(BotEntry::Alive(entry));
        }

        if let Some(entry) = self.dead.get(id) {
            return Some(BotEntry::Dead(entry));
        }

        if let Some(entry) = self.queued.get(id) {
            return Some(BotEntry::Queued(entry));
        }

        None
    }

    pub fn get_mut(&mut self, id: BotId) -> Option<BotEntryMut> {
        if let Some(entry) = self.alive.get_mut(id) {
            return Some(BotEntryMut::Alive(entry.bot));
        }

        if let Some(entry) = self.dead.get_mut(id) {
            return Some(BotEntryMut::Dead(entry));
        }

        if let Some(entry) = self.queued.get_mut(id) {
            return Some(BotEntryMut::Queued(entry));
        }

        None
    }

    pub fn contains(&self, id: BotId) -> bool {
        self.alive.contains(id)
            || self.dead.contains(id)
            || self.queued.contains(id)
    }

    pub fn remove(&mut self, id: BotId) {
        if self.alive.contains(id) {
            self.alive.remove(id);
        } else if self.dead.contains(id) {
            self.dead.remove(id);
        } else if self.queued.contains(id) {
            self.queued.remove(id);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = BotEntry> {
        let alive = self.alive.iter().map(BotEntry::Alive);
        let dead = self.dead.iter().map(BotEntry::Dead);
        let queued = self.queued.iter().map(BotEntry::Queued);

        alive.chain(dead).chain(queued)
    }
}

#[derive(Debug)]
pub enum BotEntry<'a> {
    Alive(AliveBotEntry<'a>),
    Dead(DeadBotEntry),
    Queued(QueuedBotEntry<'a>),
}

#[derive(Debug)]
pub enum BotEntryMut<'a> {
    Alive(&'a mut AliveBot),
    Dead(&'a mut DeadBot),
    Queued(&'a mut QueuedBot),
}

#[derive(Debug)]
pub struct KillBot {
    pub id: BotId,
    pub reason: String,
    pub killer: Option<BotId>,
}
