use crate::{Object, ObjectId};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BotInventory {
    objects: VecDeque<BotInventoryObject>,
}

impl BotInventory {
    pub const SIZE: usize = 32;

    pub fn add(&mut self, id: ObjectId, obj: Object) -> Result<(), ()> {
        if self.objects.len() < Self::SIZE {
            self.objects.push_front(BotInventoryObject { id, obj });

            Ok(())
        } else {
            Err(())
        }
    }

    pub fn take(&mut self, idx: u8) -> Option<(ObjectId, Object)> {
        self.objects
            .remove(idx as usize)
            .map(|obj| (obj.id, obj.obj))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct BotInventoryObject {
    id: ObjectId,

    #[serde(flatten)]
    obj: Object,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let mut target = BotInventory::default();

        for idx in 1..=32 {
            target
                .add(ObjectId::new(idx), Object::new(idx as u8))
                .unwrap();
        }

        target
            .add(ObjectId::new(255), Object::new(255))
            .unwrap_err();

        assert_eq!(32, target.take(0).unwrap().1.kind);
        assert_eq!(31, target.take(0).unwrap().1.kind);
        assert_eq!(30, target.take(0).unwrap().1.kind);

        target.add(ObjectId::new(255), Object::new(255)).unwrap();

        assert_eq!(255, target.take(0).unwrap().1.kind);
        assert_eq!(1, target.take(28).unwrap().1.kind);
    }
}
