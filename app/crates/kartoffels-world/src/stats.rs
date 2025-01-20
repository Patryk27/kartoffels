use crate::{BotId, Bots, Clock, Lives};
use ahash::AHashMap;
use bevy_ecs::system::{Local, Res, ResMut, Resource};
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone, Debug, Default, Resource)]
pub struct Stats {
    pub entries: Arc<AHashMap<BotId, BotStats>>,
}

pub fn update(
    mut stats: ResMut<Stats>,
    bots: Res<Bots>,
    lives: Res<Lives>,
    mut prev_run_at: Local<Option<Instant>>,
) {
    if prev_run_at.map_or(false, |run| run.elapsed().as_secs() < 1) {
        return;
    }

    let entries = Arc::make_mut(&mut stats.entries);

    *entries = lives
        .entries
        .iter()
        .map(|(id, lives)| {
            let ages = {
                let mut ages: BotStatsPart = lives
                    .iter()
                    .filter_map(|life| {
                        life.age.or_else(|| {
                            bots.alive.get(*id).map(|bot| bot.age())
                        })
                    })
                    .map(|age| age.ticks() as u32)
                    .collect();

                ages.sum /= Clock::HZ;
                ages.avg /= Clock::HZ as f32;
                ages.min /= Clock::HZ;
                ages.max /= Clock::HZ;
                ages
            };

            let scores = lives.iter().map(|life| life.score).collect();

            (*id, BotStats { ages, scores })
        })
        .collect();

    *prev_run_at = Some(Instant::now());
}

#[derive(Clone, Debug)]
pub struct BotStats {
    pub ages: BotStatsPart,
    pub scores: BotStatsPart,
}

#[derive(Clone, Debug, Default)]
pub struct BotStatsPart {
    pub len: u32,
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
        let mut this = Self {
            min: u32::MAX,
            ..Self::default()
        };

        for item in items {
            this.len += 1;
            this.sum += item;
            this.avg += item as f32;
            this.min = this.min.min(item);
            this.max = this.max.max(item);
        }

        if this.len == 0 {
            this.min = 0;
        } else {
            this.avg /= this.len as f32;
        }

        this
    }
}
