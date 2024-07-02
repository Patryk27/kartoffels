use crate::{AliveBot, Dir};
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BotMotor {
    pub dir: Dir,
    pub vel: u8,
    pub cooldown: u32,
}

impl BotMotor {
    pub fn new(rng: &mut impl RngCore) -> Self {
        Self {
            dir: rng.gen(),
            vel: 0,
            cooldown: 0,
        }
    }

    pub fn tick(&mut self) {
        self.cooldown = self.cooldown.saturating_sub(1);
    }

    pub fn mmio_load(&self, addr: u32) -> Result<u32, ()> {
        match addr {
            AliveBot::MEM_MOTOR => Ok((self.cooldown == 0) as u32),

            _ => Err(()),
        }
    }

    pub fn mmio_store(&mut self, addr: u32, val: u32) -> Result<(), ()> {
        match addr {
            AliveBot::MEM_MOTOR => {
                if self.cooldown == 0 && val > 0 {
                    self.vel = 1;
                    self.cooldown = 15000;
                }

                Ok(())
            }

            const { AliveBot::MEM_MOTOR + 4 } => {
                if self.cooldown == 0 {
                    let val = val as i32;

                    #[allow(clippy::comparison_chain)]
                    if val < 0 {
                        self.dir = self.dir.turned_left();
                        self.cooldown = 10000;
                    } else if val > 0 {
                        self.dir = self.dir.turned_right();
                        self.cooldown = 10000;
                    }
                }

                Ok(())
            }

            _ => Err(()),
        }
    }
}

#[cfg(test)]
impl Default for BotMotor {
    fn default() -> Self {
        Self {
            dir: Dir::Up,
            vel: Default::default(),
            cooldown: Default::default(),
        }
    }
}
