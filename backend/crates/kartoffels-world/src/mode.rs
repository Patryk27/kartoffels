mod deathmatch;

pub use self::deathmatch::*;
use crate::BotId;
use ahash::AHashMap;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Mode {
    #[serde(rename = "deathmatch")]
    Deathmatch(DeathmatchMode),
}

impl Mode {
    pub fn create(spec: &str) -> Result<Self> {
        if spec == "deathmatch" {
            return Ok(Mode::Deathmatch(DeathmatchMode::default()));
        }

        Err(anyhow!("unknown mode"))
    }

    pub(crate) fn scores(&self) -> &AHashMap<BotId, u32> {
        match self {
            Mode::Deathmatch(this) => this.scores(),
        }
    }

    pub(crate) fn on_bot_killed(
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
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Deathmatch(Default::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        assert_eq!(
            Mode::Deathmatch(DeathmatchMode::default()),
            Mode::create("deathmatch").unwrap(),
        );
    }
}
