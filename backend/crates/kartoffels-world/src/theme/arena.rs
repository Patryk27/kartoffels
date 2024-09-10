use crate::{Map, Tile, TileBase};
use glam::uvec2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArenaTheme {
    config: ArenaThemeConfig,
}

impl ArenaTheme {
    pub fn new(config: ArenaThemeConfig) -> Self {
        Self { config }
    }

    pub fn create_map(&self) -> Map {
        let mut map =
            Map::new(uvec2(self.config.radius, self.config.radius) * 2 + 1);

        let center = map.size() / 2;
        let radius = self.config.radius as f32;

        for y in 0..map.size().y {
            for x in 0..map.size().x {
                let pos = uvec2(x, y).as_ivec2();

                if center.as_vec2().distance(pos.as_vec2()) < radius {
                    map.set(pos, Tile::new(TileBase::FLOOR));
                }
            }
        }

        map
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArenaThemeConfig {
    pub radius: u32,
}
