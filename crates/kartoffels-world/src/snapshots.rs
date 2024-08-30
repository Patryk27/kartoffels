mod systems;

pub use self::systems::*;
use crate::{BotId, Map};
use ahash::AHashMap;
use glam::IVec2;
use itertools::Either;

#[derive(Debug)]
pub struct Snapshot {
    pub map: Map,
    pub bots: SnapshotBots,
}

#[derive(Debug)]
pub struct SnapshotBots {
    pub alive: SnapshotAliveBots,
    pub queued: SnapshotQueuedBots,
}

impl SnapshotBots {
    pub fn by_id(
        &self,
        id: BotId,
    ) -> Option<Either<&SnapshotAliveBot, &SnapshotQueuedBot>> {
        if let Some(bot) = self.alive.by_id(id) {
            return Some(Either::Left(bot));
        }

        if let Some(bot) = self.queued.by_id(id) {
            return Some(Either::Right(bot));
        }

        None
    }
}

#[derive(Debug)]
pub struct SnapshotAliveBots {
    pub entries: Vec<SnapshotAliveBot>,
    pub idx_lookup: AHashMap<BotId, u8>,
    pub idx_by_scores: Vec<(u32, u8)>,
}

impl SnapshotAliveBots {
    pub fn by_id(&self, id: BotId) -> Option<&SnapshotAliveBot> {
        self.by_idx(*self.idx_lookup.get(&id)?)
    }

    pub fn by_idx(&self, idx: u8) -> Option<&SnapshotAliveBot> {
        self.entries.get(idx as usize)
    }

    pub fn iter_sorted_by_scores(
        &self,
    ) -> impl Iterator<Item = (&SnapshotAliveBot, u32)> + '_ {
        self.idx_by_scores.iter().filter_map(|(score, idx)| {
            let bot = self.by_idx(*idx)?;

            Some((bot, *score))
        })
    }
}

#[derive(Debug)]
pub struct SnapshotAliveBot {
    pub id: BotId,
    pub pos: IVec2,
    pub serial: String,
    pub events: Vec<String>,
    pub age: u32,
}

#[derive(Debug)]
pub struct SnapshotQueuedBots {
    pub entries: AHashMap<BotId, SnapshotQueuedBot>,
}

impl SnapshotQueuedBots {
    pub fn by_id(&self, id: BotId) -> Option<&SnapshotQueuedBot> {
        self.entries.get(&id)
    }
}

#[derive(Debug)]
pub struct SnapshotQueuedBot {
    pub place: u8,
    pub requeued: bool,
}
