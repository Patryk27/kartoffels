use glam::{ivec2, IVec2};
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
        [AbsDir::N, AbsDir::E, AbsDir::S, AbsDir::W]
    }

    pub fn shuffled(rng: &mut impl RngCore) -> [Self; 4] {
        let mut dirs = Self::all();

        dirs.shuffle(rng);
        dirs
    }

    #[must_use]
    pub fn turned_left(self) -> Self {
        match self {
            AbsDir::N => AbsDir::W,
            AbsDir::E => AbsDir::N,
            AbsDir::S => AbsDir::E,
            AbsDir::W => AbsDir::S,
        }
    }

    #[must_use]
    pub fn turned_right(self) -> Self {
        match self {
            AbsDir::N => AbsDir::E,
            AbsDir::E => AbsDir::S,
            AbsDir::S => AbsDir::W,
            AbsDir::W => AbsDir::N,
        }
    }

    #[must_use]
    pub fn turned_back(self) -> Self {
        match self {
            AbsDir::N => AbsDir::S,
            AbsDir::E => AbsDir::W,
            AbsDir::S => AbsDir::N,
            AbsDir::W => AbsDir::E,
        }
    }

    #[must_use]
    pub fn as_vec(&self) -> IVec2 {
        match self {
            AbsDir::N => ivec2(0, -1),
            AbsDir::E => ivec2(1, 0),
            AbsDir::S => ivec2(0, 1),
            AbsDir::W => ivec2(-1, 0),
        }
    }

    #[must_use]
    pub fn as_caret(&self) -> char {
        match self {
            AbsDir::N => '^',
            AbsDir::E => '>',
            AbsDir::S => 'v',
            AbsDir::W => '<',
        }
    }
}

impl fmt::Display for AbsDir {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AbsDir::N => write!(f, "n"),
            AbsDir::E => write!(f, "e"),
            AbsDir::S => write!(f, "s"),
            AbsDir::W => write!(f, "w"),
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
            0 => AbsDir::N,
            1 => AbsDir::E,
            2 => AbsDir::S,
            _ => AbsDir::W,
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
    type Output = IVec2;

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
