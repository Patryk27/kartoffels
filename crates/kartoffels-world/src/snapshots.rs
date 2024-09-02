mod systems;

pub use self::systems::*;
use crate::{BotEvent, BotId, Map};
use ahash::AHashMap;
use glam::IVec2;
use itertools::Either;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Debug)]
pub struct Snapshot {
    map: Map,
    bots: SnapshotBots,
}

impl Snapshot {
    pub fn map(&self) -> &Map {
        &self.map
    }

    pub fn bots(&self) -> &SnapshotBots {
        &self.bots
    }
}

#[derive(Debug)]
pub struct SnapshotBots {
    alive: SnapshotAliveBots,
    queued: SnapshotQueuedBots,
}

impl SnapshotBots {
    pub fn alive(&self) -> &SnapshotAliveBots {
        &self.alive
    }

    pub fn queued(&self) -> &SnapshotQueuedBots {
        &self.queued
    }

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
    entries: Vec<SnapshotAliveBot>,
    idx_lookup: AHashMap<BotId, u8>,
    idx_by_scores: Vec<(u32, u8)>,
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
    pub serial: Arc<VecDeque<u32>>,
    pub events: Vec<Arc<BotEvent>>,
    pub age: u32,
}

#[derive(Debug)]
pub struct SnapshotQueuedBots {
    entries: AHashMap<BotId, SnapshotQueuedBot>,
}

impl SnapshotQueuedBots {
    pub fn by_id(&self, id: BotId) -> Option<&SnapshotQueuedBot> {
        self.entries.get(&id)
    }
}

#[derive(Debug)]
pub struct SnapshotQueuedBot {
    pub serial: Arc<VecDeque<u32>>,
    pub events: Vec<Arc<BotEvent>>,
    pub place: u8,
    pub requeued: bool,
}
