use crate::BotId;
use ahash::AHashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DeathmatchMode {
    scores: AHashMap<BotId, u32>,
}

impl DeathmatchMode {
    pub fn scores(&self) -> &AHashMap<BotId, u32> {
        &self.scores
    }

    pub fn on_bot_killed(
        &mut self,
        killed_id: BotId,
        killer_id: Option<BotId>,
    ) {
        self.scores.remove(&killed_id);

        if let Some(killer_id) = killer_id {
            *self.scores.entry(killer_id).or_default() += 1;
        }
    }
}
