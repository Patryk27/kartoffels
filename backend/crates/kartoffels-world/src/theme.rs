mod arena;
mod dungeon;

pub use self::arena::*;
pub use self::dungeon::*;
use crate::Map;
use anyhow::{anyhow, Result};
use rand::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Theme {
    #[serde(rename = "arena")]
    Arena(ArenaTheme),

    #[serde(rename = "dungeon")]
    Dungeon(DungeonTheme),
}

impl Theme {
    pub fn create(spec: &str) -> Result<Self> {
        if let Some(spec) = spec.strip_prefix("arena:") {
            return ArenaTheme::create(spec).map(Theme::Arena);
        }

        if let Some(spec) = spec.strip_prefix("dungeon:") {
            return DungeonTheme::create(spec).map(Theme::Dungeon);
        }

        Err(anyhow!("unknown theme"))
    }

    pub fn create_map(&self, rng: &mut impl RngCore) -> Result<Map> {
        match self {
            Theme::Arena(this) => Ok(this.create_map()),
            Theme::Dungeon(this) => this.create_map(rng),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::uvec2;

    #[test]
    fn create() {
        assert_eq!(
            Theme::Arena(ArenaTheme::new(123)),
            Theme::create("arena:radius=123").unwrap(),
        );

        assert_eq!(
            Theme::Dungeon(DungeonTheme::new(uvec2(12, 34))),
            Theme::create("dungeon:width=12,height=34").unwrap(),
        );
    }
}
