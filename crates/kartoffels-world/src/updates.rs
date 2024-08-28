mod systems;

pub use self::systems::*;
use crate::{BotId, Map};

#[derive(Clone, Debug)]
pub struct Update {
    pub map: Map,
    pub bots: Vec<BotUpdate>,
}

#[derive(Clone, Debug)]
pub struct BotUpdate {
    pub id: BotId,
    pub serial: String,
    pub events: Vec<String>,
    pub status: BotStatusUpdate,
}

#[derive(Clone, Debug)]
pub enum BotStatusUpdate {
    Alive { age: u32 },
    Queued { place: usize, requeued: bool },
}
