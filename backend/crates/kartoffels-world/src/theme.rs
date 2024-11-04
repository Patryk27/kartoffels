mod arena;
mod dungeon;

pub use self::arena::*;
pub use self::dungeon::*;
use crate::Map;
use anyhow::Result;
use rand::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Theme {
    #[serde(rename = "arena")]
    Arena(ArenaTheme),

    #[serde(rename = "dungeon")]
    Dungeon(DungeonTheme),
}

impl Theme {
    pub fn create_map(&self, rng: &mut impl RngCore) -> Result<Map> {
        match self {
            Theme::Arena(this) => Ok(this.create_map()),
            Theme::Dungeon(this) => this.create_map(rng),
        }
    }
}
