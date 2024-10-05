use crate::{AliveBot, AliveBotsLocator, BotMmioContext, Dir, Map, TileBase};
use glam::{ivec2, IVec2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BotRadar {
    pub payload: Vec<u8>,
    pub cooldown: u32,
    pub pending_scan: Option<BotRadarDistance>,
}

impl BotRadar {
    pub fn tick(
        &mut self,
        map: &Map,
        bots: &AliveBotsLocator,
        pos: IVec2,
        dir: Dir,
    ) {
        self.cooldown = self.cooldown.saturating_sub(1);

        if let Some(dist) = self.pending_scan.take() {
            let tile = |pos: IVec2| {
                if bots.contains(pos) {
                    TileBase::BOT
                } else {
                    map.get(pos).base
                }
            };

            let edge = dist.edge() as i32;
            let mut idx = 0;

            for dy in -edge..=edge {
                for dx in -edge..=edge {
                    self.payload[idx] =
                        tile(pos + dir.as_vec().rotate(ivec2(dx, dy).perp()));

                    idx += 1;
                }
            }
        }
    }

    pub fn mmio_load(&self, addr: u32) -> Result<u32, ()> {
        match addr {
            AliveBot::MEM_RADAR => Ok((self.cooldown == 0) as u32),

            addr if addr >= AliveBot::MEM_RADAR + 4 => {
                let idx = (addr - AliveBot::MEM_RADAR - 4) / 4;

                self.payload
                    .get(idx as usize)
                    .map(|ch| *ch as u32)
                    .ok_or(())
            }

            _ => Err(()),
        }
    }

    pub fn mmio_store(
        &mut self,
        ctxt: &mut BotMmioContext,
        addr: u32,
        val: u32,
    ) -> Result<(), ()> {
        match addr {
            AliveBot::MEM_RADAR => {
                if self.cooldown == 0
                    && let Some(dist) = BotRadarDistance::new(val)
                {
                    self.cooldown = match dist {
                        BotRadarDistance::D3 => ctxt.cooldown(10_000, 10),
                        BotRadarDistance::D5 => ctxt.cooldown(15_000, 15),
                        BotRadarDistance::D7 => ctxt.cooldown(22_000, 25),
                        BotRadarDistance::D9 => ctxt.cooldown(30_000, 30),
                    };

                    self.pending_scan = Some(dist);
                }

                Ok(())
            }

            _ => Err(()),
        }
    }
}

impl Default for BotRadar {
    fn default() -> Self {
        Self {
            payload: vec![TileBase::VOID; 9 * 9],
            cooldown: Default::default(),
            pending_scan: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BotRadarDistance {
    D3,
    D5,
    D7,
    D9,
}

impl BotRadarDistance {
    fn new(r: u32) -> Option<Self> {
        match r {
            3 => Some(Self::D3),
            5 => Some(Self::D5),
            7 => Some(Self::D7),
            9 => Some(Self::D9),
            _ => None,
        }
    }

    fn edge(self) -> u32 {
        match self {
            Self::D3 => 1,
            Self::D5 => 2,
            Self::D7 => 3,
            Self::D9 => 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AliveBots;
    use glam::uvec2;
    use indoc::indoc;
    use itertools::Itertools;
    use pretty_assertions as pa;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    impl BotRadar {
        #[track_caller]
        fn assert<const DIST: u32>(&self, expected: &str) {
            let actual = (0..DIST)
                .map(|y| {
                    (0..DIST)
                        .map(|x| {
                            let addr =
                                AliveBot::MEM_RADAR + 4 * (y * DIST + x + 1);

                            self.mmio_load(addr).unwrap()
                        })
                        .map(|ch| ch as u8 as char)
                        .join(" ")
                })
                .join("\n");

            pa::assert_eq!(expected.trim(), actual);
        }
    }

    #[test]
    fn test() {
        let map = {
            let mut map = Map::new(uvec2(7, 7));

            map.rect(ivec2(0, 0), ivec2(6, 6), TileBase::FLOOR);
            map.set(ivec2(3, 1), TileBase::FLAG);
            map.set(ivec2(3, 2), TileBase::BOT);
            map
        };

        let bots = AliveBots::default();
        let bots = bots.locator();
        let mut radar = BotRadar::default();
        let mut rng = ChaCha8Rng::from_seed(Default::default());

        let mut ctxt = BotMmioContext {
            dir: &mut Dir::N,
            rng: &mut rng,
        };

        // ---

        radar.mmio_store(&mut ctxt, AliveBot::MEM_RADAR, 3).unwrap();
        radar.cooldown = 0;
        radar.tick(&map, &bots, ivec2(3, 3), Dir::N);

        radar.assert::<3>(indoc! {"
            . @ .
            . . .
            . . .
        "});

        // ---

        radar.mmio_store(&mut ctxt, AliveBot::MEM_RADAR, 5).unwrap();
        radar.cooldown = 0;
        radar.tick(&map, &bots, ivec2(3, 3), Dir::N);

        radar.assert::<5>(indoc! {"
            . . = . .
            . . @ . .
            . . . . .
            . . . . .
            . . . . .
        "});

        // ---

        radar.mmio_store(&mut ctxt, AliveBot::MEM_RADAR, 5).unwrap();
        radar.cooldown = 0;
        radar.tick(&map, &bots, ivec2(3, 3), Dir::E);

        radar.assert::<5>(indoc! {"
            . . . . .
            . . . . .
            = @ . . .
            . . . . .
            . . . . .
        "});

        // ---

        radar.mmio_store(&mut ctxt, AliveBot::MEM_RADAR, 5).unwrap();
        radar.cooldown = 0;
        radar.tick(&map, &bots, ivec2(3, 3), Dir::W);

        radar.assert::<5>(indoc! {"
            . . . . .
            . . . . .
            . . . @ =
            . . . . .
            . . . . .
        "});

        // ---

        radar.mmio_store(&mut ctxt, AliveBot::MEM_RADAR, 5).unwrap();
        radar.cooldown = 0;
        radar.tick(&map, &bots, ivec2(3, 3), Dir::S);

        radar.assert::<5>(indoc! {"
            . . . . .
            . . . . .
            . . . . .
            . . @ . .
            . . = . .
        "});
    }
}
