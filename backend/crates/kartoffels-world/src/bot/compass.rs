use super::AliveBot;
use crate::Dir;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BotCompass {
    dir: Option<Dir>,
    next_measurement_in: u32,
}

impl BotCompass {
    pub fn tick(&mut self, dir: Dir) {
        if let Some(time) = self.next_measurement_in.checked_sub(1) {
            self.next_measurement_in = time;
        } else {
            self.dir = Some(dir);
            self.next_measurement_in = 128_000;
        }
    }

    pub fn mmio_load(&mut self, addr: u32) -> Result<u32, ()> {
        match addr {
            AliveBot::MEM_COMPASS => Ok(match self.dir.take() {
                None => 0,
                Some(Dir::N) => 0,
                Some(Dir::E) => 1,
                Some(Dir::S) => 2,
                Some(Dir::W) => 3,
            }),

            _ => Err(()),
        }
    }
}
