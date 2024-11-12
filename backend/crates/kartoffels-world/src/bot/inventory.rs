use crate::Object;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BotInventory {
    objects: [Option<Object>; Self::SIZE],
}

impl BotInventory {
    pub const SIZE: usize = 32;

    pub fn add(&mut self, object: Object) -> Result<(), Object> {
        for slot in &mut self.objects {
            if slot.is_none() {
                *slot = Some(object);
                return Ok(());
            }
        }

        Err(object)
    }

    pub fn take(&mut self, idx: u8) -> Option<Object> {
        self.objects.get_mut(idx as usize)?.take()
    }
}
