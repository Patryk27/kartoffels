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
    pub queued: QueuedBots,
    pub alive: AliveBots,
    pub dead: DeadBots,
}

impl Bots {
    pub fn get(&self, id: BotId) -> Option<BotEntry> {
        if let Some(entry) = self.queued.get(id) {
            return Some(BotEntry::Queued(entry));
        }

        if let Some(entry) = self.alive.get(id) {
            return Some(BotEntry::Alive(entry));
        }

        if self.dead.get(id).is_some() {
            return Some(BotEntry::Dead);
        }

        None
    }

    pub fn get_mut(&mut self, id: BotId) -> Option<BotEntryMut> {
        if let Some(entry) = self.queued.get_mut(id) {
            return Some(BotEntryMut::Queued(entry));
        }

        if let Some(entry) = self.alive.get_mut(id) {
            return Some(BotEntryMut::Alive(entry.bot));
        }

        if let Some(entry) = self.dead.get_mut(id) {
            return Some(BotEntryMut::Dead(entry));
        }

        None
    }

    pub fn remove(&mut self, id: BotId) {
        if self.alive.has(id) {
            self.alive.remove(id);
        } else if self.dead.has(id) {
            self.dead.remove(id);
        } else {
            // TODO handle queued bots as well
        }
    }

    pub fn has(&self, id: BotId) -> bool {
        self.alive.has(id) || self.dead.has(id) || self.queued.has(id)
    }
}

#[derive(Debug)]
pub enum BotEntry<'a> {
    Queued(QueuedBotEntry<'a>),
    Alive(AliveBotEntry<'a>),
    Dead,
}

#[derive(Debug)]
pub enum BotEntryMut<'a> {
    Queued(&'a mut QueuedBot),
    Alive(&'a mut AliveBot),
    Dead(&'a mut DeadBot),
}

#[derive(Debug)]
pub struct KillBot {
    pub id: BotId,
    pub reason: String,
    pub killer: Option<BotId>,
}
