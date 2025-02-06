use super::MapBuilder;
use crate::{spec, Map, TileKind};
use anyhow::{anyhow, Context, Error, Result};
use glam::uvec2;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArenaTheme {
    radius: u32,
}

impl ArenaTheme {
    pub fn new(radius: u32) -> Self {
        Self { radius }
    }

    pub async fn build(
        &self,
        rng: &mut impl RngCore,
        mut map: MapBuilder,
    ) -> Result<Map> {
        map.reveal(rng, {
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
        })
        .await;

        Ok(map.commit())
    }
}

impl FromStr for ArenaTheme {
    type Err = Error;

    fn from_str(spec: &str) -> Result<Self> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::FutureExt;
    use kartoffels_utils::Asserter;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn build() {
        let mut rng = ChaCha8Rng::from_seed(Default::default());

        let map = ArenaTheme::from_str("radius=16")
            .unwrap()
            .build(&mut rng, MapBuilder::detached())
            .now_or_never()
            .unwrap()
            .unwrap();

        Asserter::new("src/theme/arena/tests")
            .assert("build.txt", map.to_string());
    }
}
