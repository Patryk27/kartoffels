use crate::{Map, TileBase};
use glam::{ivec2, IVec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Corridor {
    pub anchor: IVec2,
    pub dir: CorridorDir,
    pub len: u32,
}

impl Corridor {
    pub fn render(&self, map: &mut Map) {
        match self.dir {
            CorridorDir::Horizontal => {
                for delta in 0..self.len {
                    let point = self.anchor + ivec2(delta as i32, 0);

                    map.set(point, TileBase::FLOOR);
                    map.set_if_void(point - ivec2(0, 1), TileBase::WALL_H);
                    map.set_if_void(point + ivec2(0, 1), TileBase::WALL_H);
                }
            }

            CorridorDir::Vertical => {
                for delta in 0..self.len {
                    let point = self.anchor + ivec2(0, delta as i32);

                    map.set(point, TileBase::FLOOR);
                    map.set_if_void(point - ivec2(1, 0), TileBase::WALL_V);
                    map.set_if_void(point + ivec2(1, 0), TileBase::WALL_V);
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorridorDir {
    Horizontal,
    Vertical,
}
