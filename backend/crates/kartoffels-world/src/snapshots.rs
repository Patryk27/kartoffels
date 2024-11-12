mod systems;

pub use self::systems::*;
use crate::{BotEvent, BotId, Dir, Map};
use ahash::AHashMap;
use glam::IVec2;
use itertools::{Either, Itertools};
use prettytable::{row, Table};
use std::cmp::Reverse;
use std::collections::VecDeque;
use std::fmt;
use std::sync::Arc;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    map: Map,
    display: Map,
    bots: SnapshotBots,
    version: u64,
}

impl Snapshot {
    pub fn map(&self) -> &Map {
        &self.map
    }

    pub fn display(&self) -> &Map {
        &self.display
    }

    pub fn bots(&self) -> &SnapshotBots {
        &self.bots
    }

    pub fn version(&self) -> u64 {
        self.version
    }
}

impl fmt::Display for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "# map")?;
        writeln!(f)?;
        writeln!(f, "```")?;
        writeln!(f, "{}", self.display)?;
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

#[derive(Debug, Default, PartialEq, Eq)]
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.alive.is_empty() {
            writeln!(f, "## alive")?;
            writeln!(f)?;
            writeln!(f, "{}", self.alive)?;
        }

        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
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

    pub fn iter_sorted_by_birth(
        &self,
    ) -> impl Iterator<Item = &SnapshotAliveBot> {
        self.entries
            .iter()
            .sorted_unstable_by_key(|bot| (Reverse(bot.age), bot.id))
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

    pub fn has(&self, id: BotId) -> bool {
        self.by_id(id).is_some()
    }

    pub fn has_all_of(&self, ids: &[BotId]) -> bool {
        ids.iter().all(|id| self.has(*id))
    }

    pub fn has_any_of(&self, ids: &[BotId]) -> bool {
        ids.iter().any(|id| self.has(*id))
    }
}

impl fmt::Display for SnapshotAliveBots {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut table = Table::init(vec![]);

        table.set_titles(row!["id", "pos", "dir", "age", "score"]);

        for bot in self.iter_sorted_by_scores() {
            table.add_row(row![bot.id, bot.pos, bot.dir, bot.age, bot.score]);
        }

        write!(f, "{table}")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SnapshotAliveBot {
    pub id: BotId,
    pub pos: IVec2,
    pub dir: Dir,
    pub age: u32,
    pub score: u32,
    pub serial: Arc<VecDeque<u32>>,
    pub events: Vec<Arc<BotEvent>>,
}

#[derive(Debug, Default, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
pub struct SnapshotQueuedBot {
    pub serial: Arc<VecDeque<u32>>,
    pub events: Vec<Arc<BotEvent>>,
    pub place: u8,
    pub requeued: bool,
}
