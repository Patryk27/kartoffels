use crate::{BotId, BotLives, Bots, Clock, World};
use ahash::AHashMap;
use serde::Serialize;
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub entries: Arc<AHashMap<BotId, BotStats>>,
}

#[derive(Debug)]
struct State {
    prev_run_at: Instant,
}

impl Default for State {
    fn default() -> Self {
        Self {
            prev_run_at: Instant::now(),
        }
    }
}

pub fn update(world: &mut World) {
    let state = world.states.get_mut::<State>();

    if state.prev_run_at.elapsed().as_secs() < 1 {
        return;
    }

    let entries = Arc::make_mut(&mut world.stats.entries);

    *entries = world
        .lives
        .entries
        .iter()
        .map(|(id, lives)| (*id, BotStats::new(&world.bots, lives, *id)))
        .collect();

    state.prev_run_at = Instant::now();
}

#[derive(Clone, Debug, Serialize)]
pub struct BotStats {
    pub ages: BotStatsPart,
    pub scores: BotStatsPart,
    pub lives: u32,
}

impl BotStats {
    fn new(bots: &Bots, lives: &BotLives, id: BotId) -> Self {
        let ages = {
            let mut ages: BotStatsPart = lives
                .iter()
                .filter_map(|life| {
                    life.age.or_else(|| bots.alive.get(id).map(|bot| bot.age()))
                })
                .map(|age| age.as_ticks() as u32)
                .collect();

            ages.sum /= Clock::HZ;
            ages.avg /= Clock::HZ as f32;
            ages.min /= Clock::HZ;
            ages.max /= Clock::HZ;
            ages
        };

        let scores = lives.iter().map(|life| life.score).collect();

        Self {
            ages,
            scores,
            lives: lives.len() as u32,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct BotStatsPart {
    pub sum: u32,
    pub avg: f32,
    pub min: u32,
    pub max: u32,
}

impl FromIterator<u32> for BotStatsPart {
    fn from_iter<T>(items: T) -> Self
    where
        T: IntoIterator<Item = u32>,
    {
        let mut len = 0;
        let mut sum = 0;
        let mut min = u32::MAX;
        let mut max = 0;

        for item in items {
            len += 1;
            sum += item;
            min = min.min(item);
            max = max.max(item);
        }

        if len == 0 {
            Self::default()
        } else {
            Self {
                sum,
                avg: (sum as f32) / (len as f32),
                min,
                max,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats() {
        let given =
            BotStatsPart::from_iter([10, 20, 30, 40, 50, 60, 70, 80, 90]);

        let expected = BotStatsPart {
            sum: 450,
            avg: 50.0,
            min: 10,
            max: 90,
        };

        assert_eq!(expected, given);
    }
}
