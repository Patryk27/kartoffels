use anyhow::{Error, Result};
use derivative::Derivative;
use kartoffels_utils::Id;
use rand::distributions::Standard;
use rand::prelude::Distribution;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Derivative,
)]
#[derivative(Debug = "transparent")]
pub struct BotId(Id);

impl BotId {
    pub const LENGTH: usize = Id::LENGTH;

    pub const fn new(id: u64) -> Self {
        Self(Id::new(id))
    }

    pub fn try_new(id: u64) -> Option<Self> {
        Id::try_new(id).map(Self)
    }

    pub fn get(&self) -> Id {
        self.0
    }
}

#[cfg(test)]
impl Default for BotId {
    fn default() -> Self {
        Self::new(1)
    }
}

impl Distribution<BotId> for Standard {
    fn sample<R>(&self, rng: &mut R) -> BotId
    where
        R: Rng + ?Sized,
    {
        BotId(rng.gen())
    }
}

impl FromStr for BotId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl fmt::Display for BotId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
