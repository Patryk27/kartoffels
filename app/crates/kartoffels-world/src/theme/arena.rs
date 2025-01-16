use crate::{spec, Map, TileKind};
use anyhow::{anyhow, Context, Result};
use glam::uvec2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArenaTheme {
    radius: u32,
}

impl ArenaTheme {
    pub fn new(radius: u32) -> Self {
        Self { radius }
    }

    pub fn create(spec: &str) -> Result<Self> {
        let mut radius = None;

        for entry in spec::entries(spec) {
            let entry = entry?;

            match entry.key {
                "radius" => {
                    radius = Some(entry.value()?);
                }

                key => {
                    return Err(anyhow!("unknown key: {key}"));
                }
            }
        }

        let radius = radius.context("missing key: radius")?;

        Ok(Self::new(radius))
    }

    pub fn create_map(&self) -> Map {
        let map = Map::new(uvec2(self.radius, self.radius) * 2 + 1);
        let center = map.center();
        let radius = self.radius as f32;

        map.map(|pos, tile| {
            if center.as_vec2().distance(pos.as_vec2()) < radius {
                TileKind::FLOOR.into()
            } else {
                tile
            }
        })
    }
}
