use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Derivative,
)]
#[derivative(Debug = "transparent")]
pub struct WorldName(String);

impl WorldName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

impl fmt::Display for WorldName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
