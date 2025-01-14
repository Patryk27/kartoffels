use crate::{BotId, Runs};
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
    runs: Res<Runs>,
    mut prev_run_at: Local<Option<Instant>>,
) {
    if prev_run_at.map_or(false, |run| run.elapsed().as_secs() < 1) {
        return;
    }

    let entries = Arc::make_mut(&mut stats.entries);

    *entries = runs
        .entries
        .iter()
        .map(|(id, runs)| {
            let mut scores_sum = 0;
            let mut scores_len = 0;
            let mut scores_avg = 0;
            let mut scores_max = 0;

            for run in runs.iter() {
                scores_sum += run.score;
                scores_len += 1;
                scores_avg += run.score;
                scores_max = scores_max.max(run.score);
            }

            let stats = BotStats {
                scores_sum,
                scores_len,
                scores_avg: (scores_avg as f32) / (scores_len as f32),
                scores_max,
            };

            (*id, stats)
        })
        .collect();

    *prev_run_at = Some(Instant::now());
}

#[derive(Clone, Debug)]
pub struct BotStats {
    pub scores_sum: u32,
    pub scores_len: u32,
    pub scores_avg: f32,
    pub scores_max: u32,
}
