use crate::{BotId, Event};
use ahash::AHashMap;
use bevy_ecs::event::EventReader;
use bevy_ecs::system::{ResMut, Resource};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{hash_map, VecDeque};

#[derive(Clone, Debug, Default, Serialize, Deserialize, Resource)]
pub struct Runs {
    curr: AHashMap<BotId, CurrRun>,
    past: VecDeque<PastRun>,
}

pub fn update(mut runs: ResMut<Runs>, mut events: EventReader<Event>) {
    for event in events.read() {
        match *event {
            Event::BotDied { id, .. } => {
                if let Some(run) = runs.curr.remove(&id) {
                    if runs.past.len() >= 4096 {
                        runs.past.pop_front();
                    }

                    runs.past.push_back(PastRun {
                        bot: id,
                        score: run.score,
                        spawned_at: run.spawned_at,
                        killed_at: Utc::now(),
                    });
                }
            }

            Event::BotScored { id } => match runs.curr.entry(id) {
                hash_map::Entry::Occupied(entry) => {
                    entry.into_mut().score += 1;
                }

                hash_map::Entry::Vacant(entry) => {
                    entry.insert(CurrRun {
                        score: 1,
                        spawned_at: Utc::now(),
                    });
                }
            },

            _ => (),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct CurrRun {
    score: u32,
    spawned_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct PastRun {
    bot: BotId,
    score: u32,
    spawned_at: DateTime<Utc>,
    killed_at: DateTime<Utc>,
}
