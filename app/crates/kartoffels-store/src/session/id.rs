use derivative::Derivative;
use kartoffels_utils::Id;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(
    Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Derivative,
)]
#[derivative(Debug = "transparent")]
pub struct SessionId(Id);

impl SessionId {
    pub fn new(id: Id) -> Self {
        Self(id)
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
