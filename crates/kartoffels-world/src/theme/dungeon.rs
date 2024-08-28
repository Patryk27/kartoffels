mod corridor;
mod room;

use self::corridor::*;
use self::room::*;
use crate::{Map, Tile, TileBase};
use glam::{ivec2, UVec2};
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DungeonTheme {
    config: DungeonThemeConfig,
}

impl DungeonTheme {
    pub fn new(config: DungeonThemeConfig) -> Self {
        Self { config }
    }

    pub(super) fn create_map(&self, rng: &mut impl RngCore) -> Map {
        let mut map = Map::new(Tile::new(TileBase::VOID), self.config.size);
        let rooms = self.generate_rooms(rng, &map);
        let corrs = self.generate_corridors(rng, &rooms);

        for room in rooms {
            room.render(&mut map);
        }

        for corr in corrs {
            corr.render(&mut map);
        }

        for y in 0..map.size().y {
            for x in 0..map.size().x {
                let point = ivec2(x as i32, y as i32);

                match map.get(point).base {
                    TileBase::WALL_H => {
                        if map.get(point - ivec2(0, 1)).is_floor()
                            && map.get(point + ivec2(0, 1)).is_floor()
                        {
                            map.set(point, Tile::new(TileBase::FLOOR));
                        }
                    }

                    TileBase::WALL_V => {
                        if map.get(point - ivec2(1, 0)).is_floor()
                            && map.get(point + ivec2(1, 0)).is_floor()
                        {
                            map.set(point, Tile::new(TileBase::FLOOR));
                        }
                    }

                    _ => (),
                }
            }
        }

        map
    }

    fn generate_rooms(&self, rng: &mut impl RngCore, map: &Map) -> Vec<Room> {
        let mut tries = 0;
        let mut rooms: Vec<Room> = Vec::new();

        while tries < 4096 {
            tries += 1;

            let room = {
                let size = ivec2(rng.gen_range(4..=16), rng.gen_range(4..=16));
                let min = map.rand_pos(rng);
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

    fn generate_corridors(
        &self,
        rng: &mut impl RngCore,
        rooms: &[Room],
    ) -> Vec<Corridor> {
        if rooms.len() <= 1 {
            return Default::default();
        }

        let mut corrs = Vec::new();

        for lhs_id in 0..rooms.len() {
            let mut lhs_corrs = 0;

            for _ in 0..64 {
                let rhs_id = rng.gen_range(0..rooms.len());

                if rhs_id == lhs_id {
                    continue;
                }

                let lhs = rooms[lhs_id];
                let rhs = rooms[rhs_id];

                if let Some(corr) = lhs.connect_with(rhs) {
                    lhs_corrs += 1;
                    corrs.push(corr);

                    if lhs_corrs >= 3 {
                        break;
                    }
                }
            }
        }

        corrs
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DungeonThemeConfig {
    pub size: UVec2,
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::uvec2;
    use pretty_assertions as pa;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use std::fs;

    #[test]
    fn test() {
        let mut rng = ChaCha8Rng::from_seed(Default::default());

        let actual = DungeonTheme::new(DungeonThemeConfig {
            size: uvec2(80, 60),
        })
        .create_map(&mut rng)
        .to_string();

        let expected =
            fs::read_to_string("src/theme/dungeon/tests/expected.txt").unwrap();

        pa::assert_eq!(expected, actual);
    }
}
