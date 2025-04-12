use derivative::Derivative;
use kartoffels_utils::Id;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Object {
    pub kind: u8,
    pub meta: [u8; 3],
}

impl Object {
    pub fn new(kind: u8) -> Self {
        Self {
            kind,
            meta: [0, 0, 0],
        }
    }

    pub fn name(&self) -> &'static str {
        match self.kind {
            ObjectKind::FLAG => "flag",
            ObjectKind::GEM => "gem",
            _ => "unknown object",
        }
    }
}

impl Serialize for Object {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        u32::from_be_bytes([
            self.kind,
            self.meta[0],
            self.meta[1],
            self.meta[2],
        ])
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Object {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let [b0, b1, b2, b3] = u32::deserialize(deserializer)?.to_be_bytes();

        Ok(Self {
            kind: b0,
            meta: [b1, b2, b3],
        })
    }
}

pub struct ObjectKind;

impl ObjectKind {
    pub const FLAG: u8 = b'=';
    pub const GEM: u8 = b'*';
}

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
pub struct ObjectId(pub(crate) Id);

impl ObjectId {
    pub const fn new(id: u64) -> Self {
        Self(Id::new(id))
    }

    pub fn get(&self) -> Id {
        self.0
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
