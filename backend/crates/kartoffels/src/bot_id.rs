use crate::Id;
use anyhow::Error;
use derivative::Derivative;
use rand::RngCore;
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
    pub fn new(rng: &mut impl RngCore) -> Self {
        Self(Id::new(rng))
    }
}

impl FromStr for BotId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl fmt::Display for BotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
