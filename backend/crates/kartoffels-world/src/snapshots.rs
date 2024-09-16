mod systems;

pub use self::systems::*;
use crate::{BotEvent, BotId, Dir, Map};
use ahash::AHashMap;
use glam::IVec2;
use itertools::Either;
use std::collections::VecDeque;
use std::fmt;
use std::sync::Arc;

#[derive(Debug, Default)]
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

impl fmt::Display for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let map = self
            .map
            .to_string()
            .trim_matches(|ch| ch == '\n')
            .to_string();

        writeln!(f, "# map")?;
        writeln!(f)?;
        writeln!(f, "```")?;
        writeln!(f, "{map}")?;
        writeln!(f, "```")?;

        if !self.bots.is_empty() {
            writeln!(f)?;
            writeln!(f, "# bots")?;
            writeln!(f)?;
            writeln!(f, "{}", self.bots)?;
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
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

    pub fn is_empty(&self) -> bool {
        self.alive.is_empty() && self.queued.is_empty()
    }
}

impl fmt::Display for SnapshotBots {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.alive.is_empty() {
            writeln!(f, "## alive")?;
            writeln!(f)?;
            writeln!(f, "{}", self.alive)?;
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct SnapshotAliveBots {
    entries: Vec<SnapshotAliveBot>,
    id_to_idx: AHashMap<BotId, u8>,
    idx_by_scores: Vec<u8>,
}

impl SnapshotAliveBots {
    pub fn by_id(&self, id: BotId) -> Option<&SnapshotAliveBot> {
        self.by_idx(*self.id_to_idx.get(&id)?)
    }

    pub fn by_idx(&self, idx: u8) -> Option<&SnapshotAliveBot> {
        self.entries.get(idx as usize)
    }

    pub fn iter(&self) -> impl Iterator<Item = &SnapshotAliveBot> {
        self.entries.iter()
    }

    pub fn iter_sorted_by_scores(
        &self,
    ) -> impl Iterator<Item = &SnapshotAliveBot> + '_ {
        self.idx_by_scores
            .iter()
            .filter_map(|idx| self.by_idx(*idx))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl fmt::Display for SnapshotAliveBots {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for bot in self.iter() {
            writeln!(f, "### {}", bot.id)?;
            writeln!(f)?;
            writeln!(f, "{}", bot)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct SnapshotAliveBot {
    pub id: BotId,
    pub pos: IVec2,
    pub dir: Dir,
    pub age: u32,
    pub score: u32,
    pub serial: Arc<VecDeque<u32>>,
    pub events: Vec<Arc<BotEvent>>,
}

impl fmt::Display for SnapshotAliveBot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "age: {}", self.age)?;
        writeln!(f, "score: {}", self.score)?;

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct SnapshotQueuedBots {
    entries: AHashMap<BotId, SnapshotQueuedBot>,
}

impl SnapshotQueuedBots {
    pub fn by_id(&self, id: BotId) -> Option<&SnapshotQueuedBot> {
        self.entries.get(&id)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug)]
pub struct SnapshotQueuedBot {
    pub serial: Arc<VecDeque<u32>>,
    pub events: Vec<Arc<BotEvent>>,
    pub place: u8,
    pub requeued: bool,
}
