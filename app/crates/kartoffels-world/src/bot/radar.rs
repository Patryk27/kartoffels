use crate::{AliveBot, BotMmioContext, TileKind};
use glam::{ivec2, IVec2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BotRadar {
    scan: Vec<u32>,
    cooldown: u32,
}

impl BotRadar {
    pub fn tick(&mut self) {
        self.cooldown = self.cooldown.saturating_sub(1);
    }

    pub fn mmio_load(&self, addr: u32) -> Result<u32, ()> {
        match addr {
            AliveBot::MEM_RADAR => Ok((self.cooldown == 0) as u32),

            addr if addr >= AliveBot::MEM_RADAR + 4 => {
                let idx = (addr - AliveBot::MEM_RADAR - 4) / 4;

                self.scan.get(idx as usize).copied().ok_or(())
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
        match (addr, val.to_le_bytes()) {
            (AliveBot::MEM_RADAR, [0x01, range, 0x00, 0x00])
                if let Some(range) = BotRadarRange::new(range) =>
            {
                if self.cooldown == 0 {
                    self.do_scan(ctxt, range);
                }

                Ok(())
            }

            _ => Err(()),
        }
    }

    fn do_scan(&mut self, ctxt: &mut BotMmioContext, range: BotRadarRange) {
        for y in 0..range.len() {
            for x in 0..range.len() {
                let pos = {
                    let offset = ivec2(x as i32, y as i32)
                        - IVec2::splat(range.len() as i32) / 2;

                    ctxt.pos + ctxt.dir.as_vec().rotate(offset.perp())
                };

                let out_z0;
                let out_z1;
                let out_z2;

                if let Some(bot_id) = ctxt.bots.lookup_at(pos) {
                    let bot_id = bot_id.get().get();

                    out_z0 = TileKind::BOT as u32;
                    out_z1 = (bot_id >> 32) as u32;
                    out_z2 = bot_id as u32;
                } else if let Some(object) = ctxt.objects.get_at(pos) {
                    out_z0 = object.kind as u32;
                    out_z1 = 0;
                    out_z2 = 0;
                } else {
                    out_z0 = ctxt.map.get(pos).kind as u32;
                    out_z1 = 0;
                    out_z2 = 0;
                }

                self.scan[range.idx(x, y, 0)] = out_z0;
                self.scan[range.idx(x, y, 1)] = out_z1;
                self.scan[range.idx(x, y, 2)] = out_z2;
            }
        }

        self.cooldown = range.cooldown(ctxt);
    }
}

impl Default for BotRadar {
    fn default() -> Self {
        Self {
            scan: vec![0; 3 * 9 * 9],
            cooldown: 0,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum BotRadarRange {
    D3 = 3,
    D5 = 5,
    D7 = 7,
    D9 = 9,
}

impl BotRadarRange {
    fn new(r: u8) -> Option<Self> {
        match r {
            3 => Some(Self::D3),
            5 => Some(Self::D5),
            7 => Some(Self::D7),
            9 => Some(Self::D9),
            _ => None,
        }
    }

    fn len(&self) -> u32 {
        *self as u32
    }

    fn idx(&self, x: u32, y: u32, z: u32) -> usize {
        let len = self.len();

        (z * len * len + y * len + x) as usize
    }

    fn cooldown(&self, ctxt: &mut BotMmioContext) -> u32 {
        match self {
            Self::D3 => ctxt.cooldown(10_000, 10),
            Self::D5 => ctxt.cooldown(15_000, 15),
            Self::D7 => ctxt.cooldown(22_000, 25),
            Self::D9 => ctxt.cooldown(30_000, 30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::Messages;
    use crate::{
        AliveBots, BotId, Dir, Map, Object, ObjectId, ObjectKind, Objects,
    };
    use glam::uvec2;
    use indoc::indoc;
    use itertools::Itertools;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use test_case::test_case;

    impl BotRadar {
        fn scanned_bots(&self, range: BotRadarRange) -> Vec<(BotId, IVec2)> {
            (0..range.len())
                .flat_map(|y| {
                    (0..range.len()).filter_map(move |x| {
                        let d0 =
                            self.mmio_load(range.addr(x, y, 1)).unwrap() as u64;

                        let d1 =
                            self.mmio_load(range.addr(x, y, 2)).unwrap() as u64;

                        let id = BotId::try_new((d0 << 32) | d1)?;
                        let pos = ivec2(x as i32, y as i32);

                        Some((id, pos))
                    })
                })
                .collect()
        }

        fn scanned_tiles(&self, range: BotRadarRange) -> String {
            (0..range.len())
                .map(|y| {
                    (0..range.len())
                        .map(|x| self.mmio_load(range.addr(x, y, 0)).unwrap())
                        .map(|ch| ch as u8 as char)
                        .join(" ")
                })
                .join("\n")
        }
    }

    impl BotRadarRange {
        fn addr(&self, x: u32, y: u32, z: u32) -> u32 {
            let base = AliveBot::MEM_RADAR + 4;
            let offset = 4 * self.idx(x, y, z) as u32;

            base + offset
        }
    }

    struct TestCase {
        pos: IVec2,
        dir: Dir,
        range: u8,
        expected_bots: &'static [(BotId, IVec2)],
        expected_tiles: &'static str,
        expected_cooldown: u32,
    }

    const TEST_3X3_1: TestCase = TestCase {
        pos: ivec2(3, 3),
        dir: Dir::N,
        range: 3,
        expected_bots: &[(BotId::new(112233445566778899), ivec2(1, 0))],
        expected_tiles: indoc! {"
            . @ .
            . . .
            . . .
        "},
        expected_cooldown: 9374,
    };

    const TEST_3X3_2: TestCase = TestCase {
        pos: ivec2(3, 2),
        dir: Dir::N,
        range: 3,
        expected_bots: &[(BotId::new(112233445566778899), ivec2(1, 1))],
        expected_tiles: indoc! {"
            . = .
            . @ .
            . . .
        "},
        expected_cooldown: 9374,
    };

    const TEST_3X3_3: TestCase = TestCase {
        pos: ivec2(3, 1),
        dir: Dir::N,
        range: 3,
        expected_bots: &[(BotId::new(112233445566778899), ivec2(1, 2))],
        expected_tiles: indoc! {"
            . . .
            . = .
            . @ .
        "},
        expected_cooldown: 9374,
    };

    const TEST_3X3_4: TestCase = TestCase {
        pos: ivec2(3, 0),
        dir: Dir::N,
        range: 3,
        expected_bots: &[],
        expected_tiles: indoc! {"

            . . .
            . = .
        "},
        expected_cooldown: 9374,
    };

    const TEST_5X5_N: TestCase = TestCase {
        pos: ivec2(3, 3),
        dir: Dir::N,
        range: 5,
        expected_bots: &[(BotId::new(112233445566778899), ivec2(2, 1))],
        expected_tiles: indoc! {"
            . . = . .
            . . @ . .
            . . . . .
            . . . . .
            . . . . .
        "},
        expected_cooldown: 15592,
    };

    const TEST_5X5_E: TestCase = TestCase {
        pos: ivec2(3, 3),
        dir: Dir::E,
        range: 5,
        expected_bots: &[(BotId::new(112233445566778899), ivec2(1, 2))],
        expected_tiles: indoc! {"
            . . . . .
            . . . . .
            = @ . . .
            . . . . .
            . . . . .
        "},
        expected_cooldown: 15592,
    };

    const TEST_5X5_W: TestCase = TestCase {
        pos: ivec2(3, 3),
        dir: Dir::W,
        range: 5,
        expected_bots: &[(BotId::new(112233445566778899), ivec2(3, 2))],
        expected_tiles: indoc! {"
            . . . . .
            . . . . .
            . . . @ =
            . . . . .
            . . . . .
        "},
        expected_cooldown: 15592,
    };

    const TEST_5X5_S: TestCase = TestCase {
        pos: ivec2(3, 3),
        dir: Dir::S,
        range: 5,
        expected_bots: &[(BotId::new(112233445566778899), ivec2(2, 3))],
        expected_tiles: indoc! {"
            . . . . .
            . . . . .
            . . . . .
            . . @ . .
            . . = . .
        "},
        expected_cooldown: 15592,
    };

    #[test_case(TEST_3X3_1)]
    #[test_case(TEST_3X3_2)]
    #[test_case(TEST_3X3_3)]
    #[test_case(TEST_3X3_4)]
    #[test_case(TEST_5X5_N)]
    #[test_case(TEST_5X5_E)]
    #[test_case(TEST_5X5_W)]
    #[test_case(TEST_5X5_S)]
    fn test(mut case: TestCase) {
        let map = {
            let mut map = Map::new(uvec2(7, 7));

            map.rect(ivec2(0, 0), ivec2(6, 6), TileKind::FLOOR);
            map
        };

        let objects = {
            let mut objects = Objects::default();

            objects.add(
                ObjectId::new(123),
                Object::new(ObjectKind::FLAG),
                Some(ivec2(3, 1)),
            );

            objects
        };

        let bots = {
            let mut bots = AliveBots::default();

            bots.add(AliveBot {
                id: BotId::new(112233445566778899),
                pos: ivec2(3, 2),
                ..Default::default()
            });

            bots
        };

        let mut radar = BotRadar::default();
        let mut rng = ChaCha8Rng::from_seed(Default::default());

        let mut ctxt = BotMmioContext {
            action: &mut None,
            bots: &bots,
            dir: &mut case.dir,
            map: &map,
            objects: &objects,
            pos: case.pos,
            rng: &mut rng,
            msgs: &mut Messages::default(),
        };

        radar
            .mmio_store(
                &mut ctxt,
                AliveBot::MEM_RADAR,
                u32::from_le_bytes([0x01, case.range, 0x00, 0x00]),
            )
            .unwrap();

        let range = BotRadarRange::new(case.range).unwrap();

        assert_eq!(case.expected_bots, radar.scanned_bots(range));

        assert_eq!(
            case.expected_tiles.trim(),
            radar.scanned_tiles(range).trim()
        );

        assert_eq!(case.expected_cooldown, radar.cooldown);
    }
}
