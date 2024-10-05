use glam::{ivec2, IVec2};
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{fmt, ops};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default))]
pub enum Dir {
    #[serde(rename = "^")]
    #[cfg_attr(test, default)]
    N,

    #[serde(rename = ">")]
    E,

    #[serde(rename = "<")]
    W,

    #[serde(rename = "v")]
    S,
}

impl Dir {
    pub fn all() -> impl Iterator<Item = Self> {
        [Dir::N, Dir::E, Dir::S, Dir::W].into_iter()
    }

    #[must_use]
    pub fn turned_left(self) -> Self {
        match self {
            Dir::N => Dir::W,
            Dir::E => Dir::N,
            Dir::W => Dir::S,
            Dir::S => Dir::E,
        }
    }

    #[must_use]
    pub fn turned_right(self) -> Self {
        match self {
            Dir::N => Dir::E,
            Dir::E => Dir::S,
            Dir::W => Dir::N,
            Dir::S => Dir::W,
        }
    }

    #[must_use]
    pub fn as_vec(&self) -> IVec2 {
        match self {
            Dir::N => ivec2(0, -1),
            Dir::E => ivec2(1, 0),
            Dir::S => ivec2(0, 1),
            Dir::W => ivec2(-1, 0),
        }
    }
}

impl fmt::Display for Dir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Distribution<Dir> for Standard {
    fn sample<R>(&self, rng: &mut R) -> Dir
    where
        R: Rng + ?Sized,
    {
        match rng.gen_range(0..4) {
            0 => Dir::N,
            1 => Dir::E,
            2 => Dir::S,
            _ => Dir::W,
        }
    }
}

impl From<u8> for Dir {
    fn from(value: u8) -> Self {
        match value {
            0 => Dir::N,
            1 => Dir::E,
            2 => Dir::S,
            _ => Dir::W,
        }
    }
}

impl From<Dir> for u8 {
    fn from(value: Dir) -> Self {
        match value {
            Dir::N => 0,
            Dir::E => 1,
            Dir::S => 2,
            Dir::W => 3,
        }
    }
}

impl ops::Add<Dir> for IVec2 {
    type Output = IVec2;

    fn add(self, rhs: Dir) -> Self::Output {
        self + rhs.as_vec()
    }
}
