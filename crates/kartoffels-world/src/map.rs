use glam::{uvec2, IVec2, UVec2};
use rand::{Rng, RngCore};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::fmt::Write;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Map {
    size: UVec2,
    tiles: Box<[Tile]>,
}

impl Map {
    pub fn new(tile: Tile, size: UVec2) -> Self {
        Self {
            size,
            tiles: vec![tile; (size.x * size.y) as usize].into_boxed_slice(),
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

    pub fn rand_pos(&self, rng: &mut impl RngCore) -> IVec2 {
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
            line.clear();

            for x in 0..self.size.x {
                let idx = self.pos_to_idx(uvec2(x, y).as_ivec2()).unwrap();

                _ = write!(line, "{}", self.tiles[idx].base as char);
            }

            writeln!(f, "{}", line.trim())?;
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
    pub const fn new(base: u8) -> Self {
        Self {
            base,
            meta: [0, 0, 0],
        }
    }

    pub const fn is_void(&self) -> bool {
        self.base == TileBase::VOID
    }

    pub const fn is_floor(&self) -> bool {
        self.base == TileBase::FLOOR
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