use crate::{cfg, BotId, Event, Ticks};
use ahash::AHashMap;
use bevy_ecs::event::EventReader;
use bevy_ecs::system::{ResMut, Resource};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{hash_map, VecDeque};
use std::sync::Arc;

#[derive(Clone, Debug, Default, Serialize, Deserialize, Resource)]
#[serde(transparent)]
pub struct Lives {
    pub entries: AHashMap<BotId, Arc<BotLives>>,
}

impl Lives {
    pub fn curr_score(&self, id: BotId) -> u32 {
        self.entries
            .get(&id)
            .map(|life| life.curr.score)
            .unwrap_or_default()
    }
}

pub fn update(mut lives: ResMut<Lives>, mut events: EventReader<Event>) {
    for event in events.read() {
        match *event {
            Event::BotBorn { id } => match lives.entries.entry(id) {
                hash_map::Entry::Occupied(entry) => {
                    Arc::make_mut(entry.into_mut()).curr.born_at = Utc::now();
                }

                hash_map::Entry::Vacant(entry) => {
                    entry.insert(Arc::new(BotLives {
                        curr: CurrBotLife {
                            score: 0,
                            born_at: Utc::now(),
                        },
                        prev: Default::default(),
                        len: 0,
                    }));
                }
            },

            Event::BotScored { id } => {
                lives
                    .entries
                    .get_mut(&id)
                    .map(Arc::make_mut)
                    .unwrap()
                    .on_bot_scored();
            }

            Event::BotDied { id, age } => {
                lives
                    .entries
                    .get_mut(&id)
                    .map(Arc::make_mut)
                    .unwrap()
                    .on_bot_died(age);
            }

            Event::BotDiscarded { id } => {
                lives.entries.remove(&id);
            }

            _ => (),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BotLives {
    pub curr: CurrBotLife,
    pub prev: VecDeque<PrevBotLife>,
    pub len: u32,
}

impl BotLives {
    fn on_bot_scored(&mut self) {
        self.curr.score = self.curr.score.saturating_add(1);
    }

    fn on_bot_died(&mut self, age: Ticks) {
        if self.prev.len() >= cfg::MAX_LIVES_PER_BOT {
            self.prev.pop_front();
        }

        self.prev.push_back(PrevBotLife {
            age,
            score: self.curr.score,
            born_at: self.curr.born_at,
            died_at: Utc::now(),
        });

        self.curr = Default::default();
        self.len = self.len.saturating_add(1);
    }

    pub fn iter(&self) -> impl Iterator<Item = BotLife> + '_ {
        let curr = self.curr.is_some().then_some(BotLife {
            age: None,
            score: self.curr.score,
            born_at: self.curr.born_at,
            died_at: None,
        });

        let prev = self.prev.iter().rev().map(|life| BotLife {
            age: Some(life.age),
            score: life.score,
            born_at: life.born_at,
            died_at: Some(life.died_at),
        });

        curr.into_iter().chain(prev)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BotLife {
    pub age: Option<Ticks>,
    pub score: u32,
    pub born_at: DateTime<Utc>,
    pub died_at: Option<DateTime<Utc>>,
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize,
)]
pub struct CurrBotLife {
    pub score: u32,
    pub born_at: DateTime<Utc>,
}

impl CurrBotLife {
    pub fn is_some(&self) -> bool {
        self.born_at != DateTime::<Utc>::default()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrevBotLife {
    pub age: Ticks,
    pub score: u32,
    pub born_at: DateTime<Utc>,
    pub died_at: DateTime<Utc>,
}
