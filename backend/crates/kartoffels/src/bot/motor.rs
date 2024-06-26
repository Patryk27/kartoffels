use crate::AliveBot;
use glam::{ivec2, IVec2};
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default))]
pub struct BotMotor {
    pub dir: IVec2,
    pub vel: u8,
    pub cooldown: u32,
}

impl BotMotor {
    pub fn new(rng: &mut impl RngCore) -> Self {
        let dir = match rng.gen_range(0..4) {
            0 => ivec2(-1, 0),
            1 => ivec2(1, 0),
            2 => ivec2(0, -1),
            3 => ivec2(0, 1),
            _ => unreachable!(),
        };

        Self {
            dir,
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
                        self.dir = -self.dir.perp();
                        self.cooldown = 10000;
                    } else if val > 0 {
                        self.dir = self.dir.perp();
                        self.cooldown = 10000;
                    }
                }

                Ok(())
            }

            _ => Err(()),
        }
    }
}
