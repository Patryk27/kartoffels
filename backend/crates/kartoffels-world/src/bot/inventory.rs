use crate::Object;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BotInventory {
    objects: VecDeque<Object>,
}

impl BotInventory {
    pub const SIZE: usize = 32;

    pub fn add(&mut self, object: Object) -> Result<(), ()> {
        if self.objects.len() < Self::SIZE {
            self.objects.push_front(object);

            Ok(())
        } else {
            Err(())
        }
    }

    pub fn take(&mut self, idx: u8) -> Option<Object> {
        self.objects.remove(idx as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let mut target = BotInventory::default();

        for idx in 0..32 {
            target.add(Object::new(idx as u8)).unwrap();
        }

        target.add(Object::new(255)).unwrap_err();

        assert_eq!(31, target.take(0).unwrap().kind);
        assert_eq!(30, target.take(0).unwrap().kind);
        assert_eq!(29, target.take(0).unwrap().kind);

        target.add(Object::new(255)).unwrap();

        assert_eq!(255, target.take(0).unwrap().kind);
        assert_eq!(0, target.take(28).unwrap().kind);
    }
}
