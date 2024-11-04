use crate::{Map, TileBase};
use glam::uvec2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArenaTheme {
    radius: u32,
}

impl ArenaTheme {
    pub fn new(radius: u32) -> Self {
        Self { radius }
    }

    pub fn create_map(&self) -> Map {
        let map = Map::new(uvec2(self.radius, self.radius) * 2 + 1);
        let center = map.center();
        let radius = self.radius as f32;

        map.map(|pos, tile| {
            if center.as_vec2().distance(pos.as_vec2()) < radius {
                TileBase::FLOOR.into()
            } else {
                tile
            }
        })
    }
}
