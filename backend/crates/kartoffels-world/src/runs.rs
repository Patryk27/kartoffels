use crate::{BotId, Event};
use ahash::AHashMap;
use bevy_ecs::event::EventReader;
use bevy_ecs::system::{ResMut, Resource};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{hash_map, VecDeque};
use std::sync::Arc;

#[derive(Clone, Debug, Default, Serialize, Deserialize, Resource)]
#[serde(transparent)]
pub struct Runs {
    pub entries: AHashMap<BotId, Arc<BotRuns>>,
}

impl Runs {
    pub fn score(&self, id: BotId) -> u32 {
        self.entries
            .get(&id)
            .map(|run| run.curr.score)
            .unwrap_or_default()
    }
}

pub fn update(mut runs: ResMut<Runs>, mut events: EventReader<Event>) {
    for event in events.read() {
        match *event {
            Event::BotSpawned { id } => match runs.entries.entry(id) {
                hash_map::Entry::Occupied(entry) => {
                    Arc::make_mut(entry.into_mut()).curr.spawned_at =
                        Utc::now();
                }

                hash_map::Entry::Vacant(entry) => {
                    entry.insert(Arc::new(BotRuns {
                        curr: CurrBotRun {
                            score: 0,
                            spawned_at: Utc::now(),
                        },
                        prev: Default::default(),
                    }));
                }
            },

            Event::BotScored { id } => {
                runs.entries
                    .get_mut(&id)
                    .map(Arc::make_mut)
                    .unwrap()
                    .on_bot_scored();
            }

            Event::BotDied { id, .. } => {
                runs.entries
                    .get_mut(&id)
                    .map(Arc::make_mut)
                    .unwrap()
                    .on_bot_died();
            }

            Event::BotForgotten { id } => {
                runs.entries.remove(&id);
            }

            _ => (),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BotRuns {
    pub curr: CurrBotRun,
    pub prev: VecDeque<PrevBotRun>,
}

impl BotRuns {
    fn on_bot_scored(&mut self) {
        self.curr.score += 1;
    }

    fn on_bot_died(&mut self) {
        if self.prev.len() >= 128 {
            self.prev.pop_front();
        }

        self.prev.push_back(PrevBotRun {
            score: self.curr.score,
            spawned_at: self.curr.spawned_at,
            killed_at: Utc::now(),
        });

        self.curr = Default::default();
    }
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize,
)]
pub struct CurrBotRun {
    pub score: u32,
    pub spawned_at: DateTime<Utc>,
}

impl CurrBotRun {
    pub fn is_some(&self) -> bool {
        self.spawned_at != DateTime::<Utc>::default()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrevBotRun {
    pub score: u32,
    pub spawned_at: DateTime<Utc>,
    pub killed_at: DateTime<Utc>,
}
