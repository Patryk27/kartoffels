mod systems;

pub use self::systems::*;
use crate::{BotId, Map};
use ahash::AHashMap;
use glam::IVec2;

#[derive(Debug)]
pub struct Update {
    pub map: Map,
    pub bots: UpdateBots,
}

#[derive(Debug)]
pub struct UpdateBots {
    pub list: Vec<UpdateBot>,
    pub index: AHashMap<BotId, usize>,
}

impl UpdateBots {
    pub fn by_id(&self, id: BotId) -> Option<&UpdateBot> {
        self.list.get(*self.index.get(&id)?)
    }

    pub fn by_idx(&self, idx: u8) -> Option<&UpdateBot> {
        self.list.get(idx as usize)
    }
}

#[derive(Debug)]
pub struct UpdateBot {
    pub id: BotId,
    pub pos: Option<IVec2>,
    pub serial: String,
    pub events: Vec<String>,
    pub status: UpdateBotStatus,
}

#[derive(Debug)]
pub enum UpdateBotStatus {
    Alive { age: u32 },
    Queued { place: usize, requeued: bool },
}
