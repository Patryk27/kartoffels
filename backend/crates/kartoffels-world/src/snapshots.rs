mod stream;
mod systems;

pub use self::stream::*;
pub use self::systems::*;
use crate::{
    BotEvent, BotId, BotRuns, BotStats, Clock, Dir, Map, Object, ObjectId,
};
use ahash::AHashMap;
use bevy_ecs::system::Resource;
use chrono::{DateTime, Utc};
use glam::IVec2;
use itertools::Itertools;
use prettytable::{row, Table};
use std::cmp::Reverse;
use std::collections::VecDeque;
use std::fmt;
use std::sync::Arc;
use tokio::sync::watch;

#[derive(Debug, Default)]
pub struct Snapshot {
    pub bots: BotsSnapshot,
    pub clock: Clock,
    pub map: Map,
    pub objects: ObjectsSnapshot,
    pub runs: RunsSnapshot,
    pub stats: StatsSnapshot,
    pub tiles: Map,
    pub version: u64,
}

impl fmt::Display for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "# map")?;
        writeln!(f)?;
        writeln!(f, "```")?;
        writeln!(f, "{}", self.map)?;
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
pub struct BotsSnapshot {
    pub alive: AliveBotsSnapshot,
    pub dead: DeadBotsSnapshot,
    pub queued: QueuedBotsSnapshot,
}

impl BotsSnapshot {
    pub fn has(&self, id: BotId) -> bool {
        self.get(id).is_some()
    }

    pub fn get(&self, id: BotId) -> Option<BotSnapshot> {
        if let Some(bot) = self.alive.get(id) {
            return Some(BotSnapshot::Alive(bot));
        }

        if let Some(bot) = self.dead.get(id) {
            return Some(BotSnapshot::Dead(bot));
        }

        if let Some(bot) = self.queued.get(id) {
            return Some(BotSnapshot::Queued(bot));
        }

        None
    }

    pub fn is_empty(&self) -> bool {
        self.alive.is_empty() && self.dead.is_empty() && self.queued.is_empty()
    }
}

impl fmt::Display for BotsSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.alive.is_empty() {
            writeln!(f, "## alive")?;
            writeln!(f)?;
            writeln!(f, "{}", self.alive)?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BotSnapshot<'a> {
    Alive(&'a AliveBotSnapshot),
    Dead(&'a DeadBotSnapshot),
    Queued(&'a QueuedBotSnapshot),
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct AliveBotsSnapshot {
    entries: Vec<AliveBotSnapshot>,
    id_to_idx: AHashMap<BotId, u8>,
    idx_by_scores: Vec<u8>,
}

impl AliveBotsSnapshot {
    pub fn get(&self, id: BotId) -> Option<&AliveBotSnapshot> {
        self.get_by_idx(*self.id_to_idx.get(&id)?)
    }

    pub fn get_by_idx(&self, idx: u8) -> Option<&AliveBotSnapshot> {
        self.entries.get(idx as usize)
    }

    pub fn has(&self, id: BotId) -> bool {
        self.get(id).is_some()
    }

    pub fn has_any_of(&self, ids: &[BotId]) -> bool {
        ids.iter().any(|id| self.has(*id))
    }

    pub fn iter(&self) -> impl Iterator<Item = &AliveBotSnapshot> {
        self.entries.iter()
    }

    pub fn iter_sorted_by_birth(
        &self,
    ) -> impl Iterator<Item = &AliveBotSnapshot> {
        self.entries
            .iter()
            .sorted_unstable_by_key(|bot| (Reverse(bot.age), bot.id))
    }

    pub fn iter_sorted_by_scores(
        &self,
    ) -> impl Iterator<Item = &AliveBotSnapshot> + '_ {
        self.idx_by_scores
            .iter()
            .filter_map(|idx| self.get_by_idx(*idx))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl fmt::Display for AliveBotsSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut table = Table::init(vec![]);

        table.set_titles(row!["id", "pos", "dir", "age", "score"]);

        for bot in self.iter_sorted_by_scores() {
            table.add_row(row![
                bot.id,
                bot.pos,
                bot.dir,
                bot.age_seconds(),
                bot.score
            ]);
        }

        write!(f, "{table}")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct AliveBotSnapshot {
    pub age: u32,
    pub dir: Dir,
    pub events: Arc<VecDeque<Arc<BotEvent>>>,
    pub id: BotId,
    pub pos: IVec2,
    pub score: u32, // TODO duplicated with top-level `runs`
    pub serial: Arc<VecDeque<u32>>,
}

impl AliveBotSnapshot {
    pub fn age_seconds(&self) -> u32 {
        self.age / Clock::HZ
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct DeadBotsSnapshot {
    entries: AHashMap<BotId, DeadBotSnapshot>,
}

impl DeadBotsSnapshot {
    pub fn get(&self, id: BotId) -> Option<&DeadBotSnapshot> {
        self.entries.get(&id)
    }

    pub fn has(&self, id: BotId) -> bool {
        self.get(id).is_some()
    }

    pub fn has_any_of(&self, ids: &[BotId]) -> bool {
        ids.iter().any(|id| self.has(*id))
    }

    pub fn ids(&self) -> impl Iterator<Item = BotId> + '_ {
        self.entries.keys().copied()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DeadBotSnapshot {
    pub events: Arc<VecDeque<Arc<BotEvent>>>,
    pub serial: Arc<VecDeque<u32>>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct QueuedBotsSnapshot {
    entries: AHashMap<BotId, QueuedBotSnapshot>,
}

impl QueuedBotsSnapshot {
    pub fn get(&self, id: BotId) -> Option<&QueuedBotSnapshot> {
        self.entries.get(&id)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct QueuedBotSnapshot {
    pub events: Arc<VecDeque<Arc<BotEvent>>>,
    pub place: u8,
    pub requeued: bool,
    pub serial: Arc<VecDeque<u32>>,
}

#[derive(Debug, Default)]
pub struct ObjectsSnapshot {
    objects: Vec<ObjectSnapshot>,
}

impl ObjectsSnapshot {
    pub fn iter(&self) -> impl Iterator<Item = &ObjectSnapshot> + '_ {
        self.objects.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
}

#[derive(Debug)]
pub struct ObjectSnapshot {
    pub id: ObjectId,
    pub obj: Object,
    pub pos: Option<IVec2>,
}

#[derive(Debug, Default)]
pub struct RunsSnapshot {
    entries: AHashMap<BotId, Arc<BotRuns>>,
}

impl RunsSnapshot {
    pub fn get(&self, id: BotId) -> impl Iterator<Item = BotRunSnapshot> + '_ {
        self.entries.get(&id).into_iter().flat_map(|runs| {
            runs.iter().map(|run| BotRunSnapshot {
                score: run.score,
                spawned_at: run.spawned_at,
                killed_at: run.killed_at,
            })
        })
    }
}

#[derive(Debug)]
pub struct BotRunSnapshot {
    pub score: u32,
    pub spawned_at: DateTime<Utc>,
    pub killed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Default)]
pub struct StatsSnapshot {
    entries: Arc<AHashMap<BotId, BotStats>>,
}

impl StatsSnapshot {
    pub fn get(&self, id: BotId) -> Option<BotStatsSnapshot> {
        self.entries.get(&id).map(|stats| BotStatsSnapshot {
            scores_sum: stats.scores_sum,
            scores_len: stats.scores_len,
            scores_avg: stats.scores_avg,
            scores_max: stats.scores_max,
        })
    }
}

#[derive(Debug)]
pub struct BotStatsSnapshot {
    pub scores_sum: u32,
    pub scores_len: u32,
    pub scores_avg: f32,
    pub scores_max: u32,
}

#[derive(Debug, Resource)]
pub struct Snapshots {
    pub tx: watch::Sender<Arc<Snapshot>>,
}
