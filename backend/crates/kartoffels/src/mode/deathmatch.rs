use crate::{BotId, Map, Theme};
use ahash::AHashMap;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;
use web_time::{Duration, Instant};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeathmatchMode {
    config: DeathmatchModeConfig,
    scores: AHashMap<BotId, u32>,

    #[serde(with = "crate::serde::instant_opt")]
    next_round_at: Option<Instant>,
}

impl DeathmatchMode {
    pub(crate) fn new(config: DeathmatchModeConfig) -> Self {
        let next_round_at = config.round_duration.map(|rd| Instant::now() + rd);

        Self {
            config,
            scores: Default::default(),
            next_round_at,
        }
    }

    pub(crate) fn state(&self) -> Value {
        #[derive(Debug, Serialize)]
        struct State<'a> {
            scores: &'a AHashMap<BotId, u32>,
            next_round_in: Option<u64>,
        }

        let next_round_in =
            self.next_round_at.map(|at| (at - Instant::now()).as_secs());

        serde_json::to_value(State {
            scores: &self.scores,
            next_round_in,
        })
        .unwrap()
    }

    pub(crate) fn on_bot_killed(
        &mut self,
        killed_id: BotId,
        killer_id: Option<BotId>,
    ) {
        self.scores.remove(&killed_id);

        if let Some(killer_id) = killer_id {
            *self.scores.entry(killer_id).or_default() += 1;
        }
    }

    pub(crate) fn on_after_tick(
        &mut self,
        rng: &mut impl RngCore,
        theme: &mut Theme,
        map: &mut Map,
    ) {
        if let (Some(next_round_at), Some(round_duration)) =
            (self.next_round_at, self.config.round_duration)
        {
            if Instant::now() >= next_round_at {
                info!("starting new round");

                *map = theme.create_map(rng);

                self.next_round_at = Some(Instant::now() + round_duration);
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeathmatchModeConfig {
    pub round_duration: Option<Duration>,
}
