mod stream;
mod systems;

pub use self::stream::*;
pub use self::systems::*;
use crate::{
    AbsDir, BotEvent, BotId, BotLife, BotLives, BotStats, Clock, Map, Object,
    ObjectId, Ticks,
};
use ahash::AHashMap;
use glam::IVec2;
use itertools::Itertools;
use prettytable::{row, Table};
use serde::Serialize;
use std::cmp::Reverse;
use std::collections::VecDeque;
use std::fmt;
use std::sync::Arc;
use tokio::sync::watch;

#[derive(Debug, Default, Serialize)]
pub struct Snapshot {
    pub bots: BotsSnapshot,
    pub clock: Clock,
    pub lives: LivesSnapshot,
    pub map: Map,
    pub objects: ObjectsSnapshot,
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

#[derive(Debug, Default, PartialEq, Eq, Serialize)]
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

#[derive(Clone, Copy, Debug, Serialize)]
pub enum BotSnapshot<'a> {
    Alive(&'a AliveBotSnapshot),
    Dead(&'a DeadBotSnapshot),
    Queued(&'a QueuedBotSnapshot),
}

#[derive(Debug, Default, PartialEq, Eq, Serialize)]
pub struct AliveBotsSnapshot {
    entries: Vec<AliveBotSnapshot>,
    #[serde(with = "kartoffels_utils::serde::sorted_map")]
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

    pub fn iter_by_birth(&self) -> impl Iterator<Item = &AliveBotSnapshot> {
        self.entries
            .iter()
            .sorted_unstable_by_key(|bot| (Reverse(bot.age), bot.id))
    }

    pub fn iter_by_scores(
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

        for bot in self.iter_by_scores() {
            table.add_row(row![
                bot.id,
                bot.pos,
                bot.dir,
                bot.age.as_seconds(),
                bot.score
            ]);
        }

        write!(f, "{table}")
    }
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct AliveBotSnapshot {
    pub age: Ticks,
    pub dir: AbsDir,
    pub events: Arc<VecDeque<Arc<BotEvent>>>,
    pub id: BotId,
    pub pos: IVec2,
    pub score: u32,
    pub serial: Arc<VecDeque<u32>>,
}

impl AliveBotSnapshot {
    pub fn serial(&self) -> String {
        self.serial
            .iter()
            .copied()
            .flat_map(char::from_u32)
            .collect()
    }
}

#[derive(Debug, Default, PartialEq, Eq, Serialize)]
pub struct DeadBotsSnapshot {
    #[serde(with = "kartoffels_utils::serde::sorted_map")]
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

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct DeadBotSnapshot {
    pub events: Arc<VecDeque<Arc<BotEvent>>>,
    pub serial: Arc<VecDeque<u32>>,
}

#[derive(Debug, Default, PartialEq, Eq, Serialize)]
pub struct QueuedBotsSnapshot {
    #[serde(with = "kartoffels_utils::serde::sorted_map")]
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

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct QueuedBotSnapshot {
    pub events: Arc<VecDeque<Arc<BotEvent>>>,
    pub place: u8,
    pub reincarnated: bool,
    pub serial: Arc<VecDeque<u32>>,
}

#[derive(Debug, Default, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct ObjectSnapshot {
    pub id: ObjectId,
    pub obj: Object,
    pub pos: Option<IVec2>,
}

#[derive(Debug, Default, Serialize)]
pub struct LivesSnapshot {
    #[serde(with = "kartoffels_utils::serde::sorted_map")]
    entries: AHashMap<BotId, Arc<BotLives>>,
}

impl LivesSnapshot {
    pub fn get(&self, id: BotId) -> Option<&BotLives> {
        Some(self.entries.get(&id)?)
    }

    pub fn iter(&self, id: BotId) -> impl Iterator<Item = BotLife> + '_ {
        self.get(id).into_iter().flat_map(|lives| lives.iter())
    }

    pub fn len(&self, id: BotId) -> u32 {
        self.entries
            .get(&id)
            .map(|entry| entry.len)
            .unwrap_or_default()
    }
}

#[derive(Debug, Default, Serialize)]
pub struct StatsSnapshot {
    #[serde(with = "kartoffels_utils::serde::sorted_map")]
    entries: Arc<AHashMap<BotId, BotStats>>,
}

impl StatsSnapshot {
    pub fn get(&self, id: BotId) -> Option<&BotStats> {
        self.entries.get(&id)
    }
}

#[derive(Debug)]
pub struct Snapshots {
    pub tx: watch::Sender<Arc<Snapshot>>,
}
