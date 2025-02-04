mod arena;
mod dungeon;
mod wfc;

pub use self::arena::*;
pub use self::dungeon::*;
pub use self::wfc::*;
use crate::Map;
use anyhow::{anyhow, Error, Result};
use bevy_ecs::system::Resource;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Resource)]
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

impl FromStr for Theme {
    type Err = Error;

    fn from_str(spec: &str) -> Result<Self> {
        if let Some(spec) = spec.strip_prefix("arena:") {
            return ArenaTheme::from_str(spec).map(Theme::Arena);
        }

        if let Some(spec) = spec.strip_prefix("dungeon:") {
            return DungeonTheme::from_str(spec).map(Theme::Dungeon);
        }

        Err(anyhow!("unknown theme"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::uvec2;

    #[test]
    fn from_str() {
        assert_eq!(
            Theme::Arena(ArenaTheme::new(123)),
            Theme::from_str("arena:radius=123").unwrap(),
        );

        assert_eq!(
            Theme::Dungeon(DungeonTheme::new(uvec2(12, 34))),
            Theme::from_str("dungeon:width=12,height=34").unwrap(),
        );
    }
}
