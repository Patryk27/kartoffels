use crate::{BotId, Event};
use ahash::AHashMap;
use bevy_ecs::event::EventReader;
use bevy_ecs::system::{ResMut, Resource};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, Resource)]
pub struct Scores {
    pub averages: AHashMap<BotId, AverageScore>,
    pub sums: AHashMap<BotId, u32>,
}

impl Scores {
    pub fn get(&self, id: BotId) -> u32 {
        self.sums.get(&id).copied().unwrap_or_default()
    }
}

pub fn update(mut scores: ResMut<Scores>, mut events: EventReader<Event>) {
    for event in events.read() {
        match *event {
            Event::BotDied { id, .. } => {
                scores.averages.entry(id).or_default().runs += 1;
                scores.sums.remove(&id);
            }

            Event::BotScored { id } => {
                scores.averages.entry(id).or_default().scores += 1;
                *scores.sums.entry(id).or_default() += 1;
            }

            _ => (),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct AverageScore {
    runs: u32,
    scores: u32,
}
