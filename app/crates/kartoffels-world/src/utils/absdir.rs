use glam::{IVec2, ivec2};
use rand::distributions::Standard;
use rand::prelude::{Distribution, SliceRandom};
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};
use std::{fmt, ops};

/// Absolute direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default))]
pub enum AbsDir {
    #[serde(rename = "^")]
    #[cfg_attr(test, default)]
    N,

    #[serde(rename = ">")]
    E,

    #[serde(rename = "v")]
    S,

    #[serde(rename = "<")]
    W,
}

impl AbsDir {
    pub fn all() -> [Self; 4] {
        [Self::N, Self::E, Self::S, Self::W]
    }

    pub fn shuffled(rng: &mut impl RngCore) -> [Self; 4] {
        let mut dirs = Self::all();

        dirs.shuffle(rng);
        dirs
    }

    #[must_use]
    pub fn turned_left(self) -> Self {
        match self {
            Self::N => Self::W,
            Self::E => Self::N,
            Self::S => Self::E,
            Self::W => Self::S,
        }
    }

    #[must_use]
    pub fn turned_right(self) -> Self {
        match self {
            Self::N => Self::E,
            Self::E => Self::S,
            Self::S => Self::W,
            Self::W => Self::N,
        }
    }

    #[must_use]
    pub fn turned_back(self) -> Self {
        match self {
            Self::N => Self::S,
            Self::E => Self::W,
            Self::S => Self::N,
            Self::W => Self::E,
        }
    }

    #[must_use]
    pub fn as_vec(&self) -> IVec2 {
        match self {
            Self::N => ivec2(0, -1),
            Self::E => ivec2(1, 0),
            Self::S => ivec2(0, 1),
            Self::W => ivec2(-1, 0),
        }
    }

    #[must_use]
    pub fn as_caret(&self) -> char {
        match self {
            Self::N => '^',
            Self::E => '>',
            Self::S => 'v',
            Self::W => '<',
        }
    }
}

impl fmt::Display for AbsDir {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::N => write!(f, "n"),
            Self::E => write!(f, "e"),
            Self::S => write!(f, "s"),
            Self::W => write!(f, "w"),
        }
    }
}

impl Distribution<AbsDir> for Standard {
    fn sample<R>(&self, rng: &mut R) -> AbsDir
    where
        R: Rng + ?Sized,
    {
        match rng.gen_range(0..4) {
            0 => AbsDir::N,
            1 => AbsDir::E,
            2 => AbsDir::S,
            _ => AbsDir::W,
        }
    }
}

impl From<u8> for AbsDir {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::N,
            1 => Self::E,
            2 => Self::S,
            _ => Self::W,
        }
    }
}

impl From<AbsDir> for u8 {
    fn from(value: AbsDir) -> Self {
        match value {
            AbsDir::N => 0,
            AbsDir::E => 1,
            AbsDir::S => 2,
            AbsDir::W => 3,
        }
    }
}

impl ops::Add<AbsDir> for IVec2 {
    type Output = Self;

    fn add(self, rhs: AbsDir) -> Self::Output {
        self + rhs.as_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(AbsDir::N, ivec2(0, -1))]
    #[test_case(AbsDir::E, ivec2(1, 0))]
    #[test_case(AbsDir::S, ivec2(0, 1))]
    #[test_case(AbsDir::W, ivec2(-1, 0))]
    fn as_vec(lhs: AbsDir, rhs: IVec2) {
        assert_eq!(lhs.as_vec(), rhs);
    }

    #[test_case(AbsDir::N, '^')]
    #[test_case(AbsDir::E, '>')]
    #[test_case(AbsDir::S, 'v')]
    #[test_case(AbsDir::W, '<')]
    fn as_caret(lhs: AbsDir, rhs: char) {
        assert_eq!(lhs.as_caret(), rhs);
    }
}
