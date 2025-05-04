use derivative::Derivative;
use kartoffels_utils::Id;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(
    Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Derivative,
)]
#[derivative(Debug = "transparent")]
pub struct WorldId(Id);

impl WorldId {
    pub fn new(id: Id) -> Self {
        Self(id)
    }

    pub fn get(&self) -> Id {
        self.0
    }
}

impl fmt::Display for WorldId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
