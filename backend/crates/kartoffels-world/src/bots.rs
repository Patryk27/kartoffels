mod alive;
mod dead;
mod queued;
mod systems;

pub use self::alive::*;
pub use self::dead::*;
pub use self::queued::*;
pub use self::systems::*;
use crate::BotId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Bots {
    pub alive: AliveBots,
    pub dead: DeadBots,
    pub queued: QueuedBots,
}

impl Bots {
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
}

#[derive(Debug)]
pub struct KillBot {
    pub id: BotId,
    pub reason: String,
    pub killer: Option<BotId>,
}
