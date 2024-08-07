mod deathmatch;

pub use self::deathmatch::*;
use crate::{BotId, Map, Theme};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Mode {
    #[serde(rename = "deathmatch")]
    Deathmatch(Box<DeathmatchMode>),
}

impl Mode {
    pub fn ty(&self) -> &'static str {
        match self {
            Mode::Deathmatch(_) => "deathmatch",
        }
    }

    pub fn state(&self) -> Value {
        match self {
            Mode::Deathmatch(this) => this.state(),
        }
    }

    pub fn on_bot_killed(
        &mut self,
        killed_id: BotId,
        killer_id: Option<BotId>,
    ) {
        match self {
            Mode::Deathmatch(this) => {
                this.on_bot_killed(killed_id, killer_id);
            }
        }
    }

    pub fn on_after_tick(
        &mut self,
        rng: &mut impl RngCore,
        theme: &mut Theme,
        map: &mut Map,
    ) {
        match self {
            Mode::Deathmatch(this) => {
                this.on_after_tick(rng, theme, map);
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ModeConfig {
    #[serde(rename = "deathmatch")]
    Deathmatch(DeathmatchModeConfig),
}

impl ModeConfig {
    pub(crate) fn create(self) -> Mode {
        match self {
            ModeConfig::Deathmatch(config) => {
                Mode::Deathmatch(Box::new(DeathmatchMode::new(config)))
            }
        }
    }
}
