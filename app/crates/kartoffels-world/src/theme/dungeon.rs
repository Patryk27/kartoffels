mod corridor;
mod room;

use self::corridor::*;
use self::room::*;
use crate::{spec, Dir, Map, Tile, TileKind};
use anyhow::{anyhow, Context, Result};
use glam::{ivec2, uvec2, UVec2};
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DungeonTheme {
    size: UVec2,
}

impl DungeonTheme {
    pub fn new(size: UVec2) -> Self {
        Self { size }
    }

    pub fn create(spec: &str) -> Result<Self> {
        let mut width = None;
        let mut height = None;

        for entry in spec::entries(spec) {
            let entry = entry?;

            match entry.key {
                "width" => {
                    width = Some(entry.value()?);
                }

                "height" => {
                    height = Some(entry.value()?);
                }

                key => {
                    return Err(anyhow!("unknown key: {key}"));
                }
            }
        }

        let width = width.context("missing key: width")?;
        let height = height.context("missing key: height")?;

        Ok(Self::new(uvec2(width, height)))
    }

    pub fn create_map(&self, rng: &mut impl RngCore) -> Result<Map> {
        let min_occupied_tiles = self.size.element_product() / 4;

        for _ in 0..128 {
            let mut map = Map::new(self.size);
            let rooms = self.generate_rooms(rng, &map);
            let corrs = self.generate_corridors(&rooms);

            self.render_features(&mut map, rooms, corrs);

            self.remove_unreachable_features(rng, &mut map)
                .context("couldn't remove unreachable features")?;

            if self.count_occupied_tiles(&map) < min_occupied_tiles {
                continue;
            }

            return Ok(map);
        }

        Err(anyhow!(
            "couldn't generate a valid dungeon within the time limit"
        ))
    }

    fn generate_rooms(&self, rng: &mut impl RngCore, map: &Map) -> Vec<Room> {
        let mut tries = 0;
        let mut rooms: Vec<Room> = Vec::new();

        while tries < 4096 {
            tries += 1;

            let room = {
                let size = ivec2(rng.gen_range(4..=20), rng.gen_range(4..=10));
                let min = map.sample_pos(rng);
                let max = min + size;

                Room { min, max }
            };

            if !map.contains(room.min) || !map.contains(room.max) {
                continue;
            }

            if rooms
                .iter()
                .any(|other_room| other_room.collides_with(room, 3))
            {
                continue;
            }

            rooms.push(room);
        }

        rooms
    }

    fn generate_corridors(&self, rooms: &[Room]) -> Vec<Corridor> {
        let mut corrs = Vec::new();

        for lhs_id in 0..rooms.len() {
            for rhs_id in (lhs_id + 1)..rooms.len() {
                let lhs = rooms[lhs_id];
                let rhs = rooms[rhs_id];

                if let Some(corr) = lhs.connect_with(rhs) {
                    corrs.push(corr);
                }
            }
        }

        corrs
    }

    fn render_features(
        &self,
        map: &mut Map,
        rooms: Vec<Room>,
        corrs: Vec<Corridor>,
    ) {
        for room in rooms {
            room.render(map);
        }

        for corr in corrs {
            corr.render(map);
        }
    }

    fn remove_unreachable_features(
        &self,
        rng: &mut impl RngCore,
        map: &mut Map,
    ) -> Result<()> {
        const NOT_VISITED: u8 = 0;
        const VISITED: u8 = 1;

        let room_pos = (0..1024)
            .find_map(|_| {
                let pos = map.sample_pos(rng);

                if map.get(pos).is_floor() {
                    Some(pos)
                } else {
                    None
                }
            })
            .context("couldn't find any room")?;

        map.set(
            room_pos,
            Tile {
                kind: TileKind::FLOOR,
                meta: [VISITED, 0, 0],
            },
        );

        let mut stack = VecDeque::from_iter([room_pos]);

        while let Some(pos) = stack.pop_front() {
            for dir in Dir::all() {
                let pos = pos + dir;
                let tile = map.get(pos);

                if tile.is_floor() && tile.meta[0] == NOT_VISITED {
                    map.set(
                        pos,
                        Tile {
                            kind: TileKind::FLOOR,
                            meta: [VISITED, 0, 0],
                        },
                    );

                    stack.push_back(pos);
                }
            }
        }

        // ---

        map.for_each_mut(|_, tile| {
            if tile.is_floor() && tile.meta[0] == NOT_VISITED {
                *tile = TileKind::VOID.into();
            }
        });

        for y in 0..map.size().y {
            for x in 0..map.size().x {
                let pos = ivec2(x as i32, y as i32);
                let tile = map.get(pos);

                if tile.is_wall() {
                    let mut has_floor_nearby = false;

                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            has_floor_nearby |=
                                map.get(pos + ivec2(dx, dy)).is_floor();
                        }
                    }

                    if !has_floor_nearby {
                        map.set(pos, TileKind::VOID);
                    }
                }
            }
        }

        map.for_each_mut(|_, tile| {
            tile.meta[0] = 0;
        });

        Ok(())
    }

    fn count_occupied_tiles(&self, map: &Map) -> u32 {
        let mut tiles = 0;

        map.for_each(|_, tile| {
            if !tile.is_void() {
                tiles += 1;
            }
        });

        tiles
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::uvec2;
    use kartoffels_utils::Asserter;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use std::path::Path;
    use test_case::test_case;

    #[test_case("small-1", uvec2(40, 20))]
    #[test_case("small-2", uvec2(20, 40))]
    #[test_case("medium", uvec2(80, 60))]
    #[test_case("large", uvec2(128, 128))]
    #[test_case("huge", uvec2(256, 256))]
    fn test(case: &str, size: UVec2) {
        let dir = Path::new("src").join("theme").join("dungeon").join("tests");

        let mut rng = ChaCha8Rng::from_seed(Default::default());

        let actual = DungeonTheme::new(size)
            .create_map(&mut rng)
            .unwrap()
            .to_string();

        Asserter::new(dir).assert(format!("{case}.txt"), actual);
    }
}
