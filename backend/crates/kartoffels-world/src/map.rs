use glam::{ivec2, uvec2, IVec2, UVec2};
use rand::{Rng, RngCore};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Write;
use std::{cmp, fmt};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Map {
    size: UVec2,
    tiles: Box<[Tile]>,
}

impl Map {
    pub fn new(size: UVec2) -> Self {
        Self {
            size,
            tiles: vec![Tile::new(TileBase::VOID); (size.x * size.y) as usize]
                .into_boxed_slice(),
        }
    }

    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub fn center(&self) -> IVec2 {
        self.size.as_ivec2() / 2
    }

    pub fn get(&self, pos: IVec2) -> Tile {
        if let Some(idx) = self.pos_to_idx(pos) {
            self.tiles[idx]
        } else {
            Tile::new(TileBase::VOID)
        }
    }

    pub fn set(&mut self, pos: IVec2, tile: Tile) {
        if let Some(idx) = self.pos_to_idx(pos) {
            self.tiles[idx] = tile;
        }
    }

    pub fn set_if_void(&mut self, point: IVec2, tile: Tile) {
        if self.get(point).is_void() {
            self.set(point, tile);
        }
    }

    pub fn line(&mut self, p1: IVec2, p2: IVec2, tile: Tile) {
        if p1.x == p2.x {
            let [y1, y2] = cmp::minmax(p1.y, p2.y);

            for y in y1..=y2 {
                self.set(ivec2(p1.x, y), tile);
            }
        } else if p1.y == p2.y {
            let [x1, x2] = cmp::minmax(p1.x, p2.x);

            for x in x1..=x2 {
                self.set(ivec2(x, p1.y), tile);
            }
        } else {
            unimplemented!();
        }
    }

    pub fn poly(
        &mut self,
        points: impl IntoIterator<Item = IVec2>,
        tile: Tile,
    ) {
        let mut prev = None;

        for p2 in points {
            if let Some(p1) = prev.replace(p2) {
                self.line(p1, p2, tile);
            }
        }
    }

    pub fn rect(&mut self, p1: IVec2, p2: IVec2, tile: Tile) {
        let min = p1.min(p2);
        let max = p1.max(p2);

        for y in min.y..=max.y {
            for x in min.x..=max.x {
                self.set(ivec2(x, y), tile);
            }
        }
    }

    pub fn sample_pos(&self, rng: &mut impl RngCore) -> IVec2 {
        uvec2(rng.gen_range(0..self.size.x), rng.gen_range(0..self.size.y))
            .as_ivec2()
    }

    pub fn contains(&self, pos: IVec2) -> bool {
        self.pos_to_idx(pos).is_some()
    }

    fn pos_to_idx(&self, pos: IVec2) -> Option<usize> {
        let size = self.size.as_ivec2();

        if pos.x >= 0 && pos.x < size.x && pos.y >= 0 && pos.y < size.y {
            Some((pos.x + pos.y * size.x) as usize)
        } else {
            None
        }
    }
}

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut line = String::new();

        for y in 0..self.size.y {
            if y > 0 {
                writeln!(f)?;
            }

            line.clear();

            for x in 0..self.size.x {
                let idx = self.pos_to_idx(uvec2(x, y).as_ivec2()).unwrap();

                _ = write!(line, "{}", self.tiles[idx].base as char);
            }

            write!(f, "{}", line.trim_end())?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(packed)]
pub struct Tile {
    pub base: u8,
    pub meta: [u8; 3],
}

impl Tile {
    pub fn new(base: u8) -> Self {
        Self {
            base,
            meta: [0, 0, 0],
        }
    }

    pub fn is_void(&self) -> bool {
        self.base == TileBase::VOID
    }

    pub fn is_floor(&self) -> bool {
        self.base == TileBase::FLOOR
    }

    pub fn is_wall(&self) -> bool {
        self.base == TileBase::WALL_H || self.base == TileBase::WALL_V
    }

    pub fn is_bot(&self) -> bool {
        self.base == TileBase::BOT
    }
}

impl Serialize for Tile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        u32::from_be_bytes([
            self.base,
            self.meta[0],
            self.meta[1],
            self.meta[2],
        ])
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Tile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let [b0, b1, b2, b3] = u32::deserialize(deserializer)?.to_be_bytes();

        Ok(Self {
            base: b0,
            meta: [b1, b2, b3],
        })
    }
}

pub struct TileBase;

impl TileBase {
    pub const UNKNOWN: u8 = 0;
    pub const VOID: u8 = b' ';
    pub const FLOOR: u8 = b'.';
    pub const WALL_H: u8 = b'-';
    pub const WALL_V: u8 = b'|';
    pub const FLAG: u8 = b'=';
    pub const BOT: u8 = b'@';
    pub const BOT_CHEVRON: u8 = b'~';
}
