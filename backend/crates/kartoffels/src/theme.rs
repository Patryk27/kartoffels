mod arena;
mod dungeon;

pub use self::arena::*;
pub use self::dungeon::*;
use crate::Map;
use rand::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Theme {
    #[serde(rename = "arena")]
    Arena(Box<ArenaTheme>),

    #[serde(rename = "dungeon")]
    Dungeon(Box<DungeonTheme>),
}

impl Theme {
    pub fn ty(&self) -> &'static str {
        match self {
            Theme::Arena(_) => "arena",
            Theme::Dungeon(_) => "dungeon",
        }
    }

    pub fn create_map(&self, rng: &mut impl RngCore) -> Map {
        match self {
            Theme::Arena(this) => this.create_map(),
            Theme::Dungeon(this) => this.create_map(rng),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ThemeConfig {
    #[serde(rename = "arena")]
    Arena(ArenaThemeConfig),

    #[serde(rename = "dungeon")]
    Dungeon(DungeonThemeConfig),
}

impl ThemeConfig {
    pub(crate) fn create(self) -> Theme {
        match self {
            ThemeConfig::Arena(config) => {
                Theme::Arena(Box::new(ArenaTheme::new(config)))
            }
            ThemeConfig::Dungeon(config) => {
                Theme::Dungeon(Box::new(DungeonTheme::new(config)))
            }
        }
    }
}
