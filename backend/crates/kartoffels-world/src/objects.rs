use ahash::AHashMap;
use glam::IVec2;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Objects {
    objects: AHashMap<IVec2, Object>,
}

impl Objects {
    pub fn put(&mut self, pos: IVec2, obj: impl Into<Object>) {
        self.objects.insert(pos, obj.into());
    }

    // pub fn get(&self, pos: IVec2) -> Option<Object> {
    //     self.objects.get(&pos).copied()
    // }

    pub fn take(&mut self, pos: IVec2) -> Option<Object> {
        self.objects.remove(&pos)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(packed)]
pub struct Object {
    pub kind: u8,
    pub meta: [u8; 3],
}

impl Object {
    pub fn new(ty: u8) -> Self {
        Self {
            kind: ty,
            meta: [0, 0, 0],
        }
    }
}

impl From<u8> for Object {
    fn from(value: u8) -> Self {
        Self::new(value)
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
    pub const DIAMOND: u8 = b'^';
}
