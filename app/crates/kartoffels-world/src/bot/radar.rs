use super::AliveBotBody;
use crate::{TileKind, World};
use glam::{IVec2, IVec3, ivec2};
use kartoffel as api;
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BotRadar {
    memory: Vec<u32>,
    cooldown: u32,
}

impl BotRadar {
    const COOLDOWN_3X3: u32 = 4_000;
    const COOLDOWN_5X5: u32 = 8_000;
    const COOLDOWN_7X7: u32 = 16_000;
    const COOLDOWN_9X9: u32 = 32_000;
    const COOLDOWN_OPT_TILES: u32 = 4_000;
    const COOLDOWN_OPT_BOTS: u32 = 4_000;
    const COOLDOWN_OPT_OBJS: u32 = 4_000;
    const COOLDOWN_OPT_IDS: u32 = 8_000;
    const COOLDOWN_OPT_DIRS: u32 = 8_000;

    pub(super) fn tick(bot: &mut AliveBotBody) {
        if bot.radar.cooldown == 1 {
            bot.irq.raise(api::IRQ_RADAR_IDLE, [0x00, 0x00, 0x00]);
        }

        bot.radar.cooldown = bot.radar.cooldown.saturating_sub(1);
    }

    pub(super) fn load(bot: &AliveBotBody, addr: u32) -> Result<u32, ()> {
        match addr {
            api::RADAR_MEM => Ok((bot.radar.cooldown == 0) as u32),

            addr if (api::RADAR_MEM + 4..api::COMPASS_MEM).contains(&addr) => {
                bot.radar
                    .memory
                    .get((addr - api::RADAR_MEM - 4) as usize / 4)
                    .copied()
                    .ok_or(())
            }

            _ => Err(()),
        }
    }

    pub(super) fn store(
        bot: &mut AliveBotBody,
        world: &mut World,
        addr: u32,
        val: u32,
    ) -> Result<(), ()> {
        match (addr, val.to_le_bytes()) {
            (api::RADAR_MEM, [0x01, range, opts, addr])
                if let (Some(range), Some(addr)) =
                    (BotRadarRange::new(range), BotRadarAddr::new(addr)) =>
            {
                Self::do_scan(bot, world, range, BotRadarOpts::new(opts), addr);
                Ok(())
            }

            _ => Err(()),
        }
    }

    fn do_scan(
        bot: &mut AliveBotBody,
        world: &mut World,
        range: BotRadarRange,
        opts: BotRadarOpts,
        addr: BotRadarAddr,
    ) {
        if bot.radar.cooldown > 0 {
            return;
        }

        for off in range.iter_2d() {
            let pos = bot.pos + bot.dir.as_vec().rotate(off.perp());
            let [z0, z1, z2] = Self::scan_one(world, opts, pos);

            bot.radar.memory[addr.idx(range, off.extend(0))] = z0;
            bot.radar.memory[addr.idx(range, off.extend(1))] = z1;
            bot.radar.memory[addr.idx(range, off.extend(2))] = z2;
        }

        bot.irq.raise(api::IRQ_RADAR_BUSY, [0x00, 0x00, 0x00]);
        bot.radar.cooldown = world.cooldown(Self::cooldown(range, opts));
    }

    fn scan_one(world: &World, opts: BotRadarOpts, pos: IVec2) -> [u32; 3] {
        if opts.bots
            && let Some(id) = world.bots.alive.lookup_at(pos)
        {
            let z0 = {
                let dir = if opts.dirs
                    && let Some(bot) = world.bots.alive.get(id)
                {
                    bot.dir.as_caret() as u8
                } else {
                    0
                };

                u32::from_le_bytes([TileKind::BOT, dir, 0, 0])
            };

            let [z1, z2] = if opts.ids {
                let id = id.get().get();

                [(id >> 32) as u32, id as u32]
            } else {
                [0, 0]
            };

            return [z0, z1, z2];
        }

        if opts.objs
            && let Some((id, obj)) = world.objects.get_at(pos)
        {
            let z0 = u32::from_le_bytes([obj.kind, 0, 0, 0]);

            let [z1, z2] = if opts.ids {
                let id = id.get().get();

                [(id >> 32) as u32, id as u32]
            } else {
                [0, 0]
            };

            return [z0, z1, z2];
        }

        if opts.tiles {
            return [world.map.get(pos).kind as u32, 0, 0];
        }

        [0, 0, 0]
    }

    fn cooldown(range: BotRadarRange, opts: BotRadarOpts) -> u32 {
        let mut val = match range {
            BotRadarRange::R3 => Self::COOLDOWN_3X3,
            BotRadarRange::R5 => Self::COOLDOWN_5X5,
            BotRadarRange::R7 => Self::COOLDOWN_7X7,
            BotRadarRange::R9 => Self::COOLDOWN_9X9,
        };

        if opts.tiles {
            val += Self::COOLDOWN_OPT_TILES;
        }
        if opts.bots {
            val += Self::COOLDOWN_OPT_BOTS;
        }
        if opts.objs {
            val += Self::COOLDOWN_OPT_OBJS;
        }
        if opts.ids {
            val += Self::COOLDOWN_OPT_IDS;
        }
        if opts.dirs {
            val += Self::COOLDOWN_OPT_DIRS;
        }

        val
    }
}

impl Default for BotRadar {
    fn default() -> Self {
        Self {
            memory: vec![0; 3 * 9 * 9],
            cooldown: 0,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum BotRadarRange {
    R3 = 3,
    R5 = 5,
    R7 = 7,
    R9 = 9,
}

impl BotRadarRange {
    fn new(r: u8) -> Option<Self> {
        match r {
            3 => Some(Self::R3),
            5 => Some(Self::R5),
            7 => Some(Self::R7),
            9 => Some(Self::R9),
            _ => None,
        }
    }

    fn len(&self) -> i32 {
        *self as i32
    }

    fn iter_1d(&self) -> RangeInclusive<i32> {
        let t = self.len() / 2;

        -t..=t
    }

    fn iter_2d(self) -> impl Iterator<Item = IVec2> {
        self.iter_1d()
            .flat_map(move |y| self.iter_1d().map(move |x| ivec2(x, y)))
    }
}

#[derive(Clone, Copy, Debug)]
struct BotRadarOpts {
    tiles: bool,
    bots: bool,
    objs: bool,
    ids: bool,
    dirs: bool,
}

impl BotRadarOpts {
    fn new(opts: u8) -> Self {
        if opts == 0 {
            Self {
                tiles: true,
                bots: true,
                objs: true,
                ids: false,
                dirs: false,
            }
        } else {
            Self {
                tiles: (opts & api::RADAR_SCAN_TILES) > 0,
                bots: (opts & api::RADAR_SCAN_BOTS) > 0,
                objs: (opts & api::RADAR_SCAN_OBJS) > 0,
                ids: (opts & api::RADAR_SCAN_IDS) > 0,
                dirs: (opts & api::RADAR_SCAN_DIRS) > 0,
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum BotRadarAddr {
    Array,
    Szudzik,
}

impl BotRadarAddr {
    fn new(addr: u8) -> Option<Self> {
        match addr {
            0 => Some(Self::Array),
            1 => Some(Self::Szudzik),
            _ => None,
        }
    }

    fn idx(&self, range: BotRadarRange, off: IVec3) -> usize {
        match self {
            BotRadarAddr::Array => {
                let len = range.len();
                let off = off + IVec2::splat(len / 2).extend(0);

                (off.z * len * len + off.y * len + off.x) as usize
            }

            BotRadarAddr::Szudzik => {
                api::radar_idx(off.x, off.y, off.z as u8) as usize
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AbsDir, AliveBot, BotId, Map, Object, ObjectId, ObjectKind};
    use glam::{ivec3, uvec2};
    use indoc::indoc;
    use itertools::Itertools;
    use kartoffels_utils::Id;
    use test_case::test_case;

    struct TestCase {
        pos: IVec2,
        dir: AbsDir,
        range: u8,
        opts: u8,
        expected_map: &'static str,
        expected_ids: &'static [(IVec2, Id)],
        expected_dirs: &'static [(IVec2, char)],
        expected_cooldown: u32,
    }

    const TEST_3X3_1: TestCase = TestCase {
        pos: ivec2(3, 3),
        dir: AbsDir::N,
        range: 3,
        opts: 0,
        expected_map: indoc! {"
            . @ .
            . . .
            . . .
        "},
        expected_ids: &[],
        expected_dirs: &[],
        expected_cooldown: BotRadar::COOLDOWN_3X3
            + BotRadar::COOLDOWN_OPT_TILES
            + BotRadar::COOLDOWN_OPT_BOTS
            + BotRadar::COOLDOWN_OPT_OBJS,
    };

    const TEST_3X3_2: TestCase = TestCase {
        pos: ivec2(3, 2),
        dir: AbsDir::N,
        range: 3,
        opts: 0,
        expected_map: indoc! {"
            . = .
            . @ .
            . . .
        "},
        expected_ids: &[],
        expected_dirs: &[],
        expected_cooldown: BotRadar::COOLDOWN_3X3
            + BotRadar::COOLDOWN_OPT_TILES
            + BotRadar::COOLDOWN_OPT_BOTS
            + BotRadar::COOLDOWN_OPT_OBJS,
    };

    const TEST_3X3_3: TestCase = TestCase {
        pos: ivec2(3, 1),
        dir: AbsDir::N,
        range: 3,
        opts: 0,
        expected_map: indoc! {"
            . . .
            . = .
            . @ .
        "},
        expected_ids: &[],
        expected_dirs: &[],
        expected_cooldown: BotRadar::COOLDOWN_3X3
            + BotRadar::COOLDOWN_OPT_TILES
            + BotRadar::COOLDOWN_OPT_BOTS
            + BotRadar::COOLDOWN_OPT_OBJS,
    };

    const TEST_3X3_4: TestCase = TestCase {
        pos: ivec2(3, 0),
        dir: AbsDir::N,
        range: 3,
        opts: 0,
        expected_map: indoc! {"

            . . .
            . = .
        "},
        expected_ids: &[],
        expected_dirs: &[],
        expected_cooldown: BotRadar::COOLDOWN_3X3
            + BotRadar::COOLDOWN_OPT_TILES
            + BotRadar::COOLDOWN_OPT_BOTS
            + BotRadar::COOLDOWN_OPT_OBJS,
    };

    const TEST_5X5_N: TestCase = TestCase {
        pos: ivec2(3, 3),
        dir: AbsDir::N,
        range: 5,
        opts: 0,
        expected_map: indoc! {"
            . . = . .
            . . @ . .
            . . . . .
            . . . . .
            . . . . .
        "},
        expected_ids: &[],
        expected_dirs: &[],
        expected_cooldown: BotRadar::COOLDOWN_5X5
            + BotRadar::COOLDOWN_OPT_TILES
            + BotRadar::COOLDOWN_OPT_BOTS
            + BotRadar::COOLDOWN_OPT_OBJS,
    };

    const TEST_5X5_E: TestCase = TestCase {
        pos: ivec2(3, 3),
        dir: AbsDir::E,
        range: 5,
        opts: 0,
        expected_map: indoc! {"
            . . . . .
            . . . . .
            = @ . . .
            . . . . .
            . . . . .
        "},
        expected_ids: &[],
        expected_dirs: &[],
        expected_cooldown: BotRadar::COOLDOWN_5X5
            + BotRadar::COOLDOWN_OPT_TILES
            + BotRadar::COOLDOWN_OPT_BOTS
            + BotRadar::COOLDOWN_OPT_OBJS,
    };

    const TEST_5X5_W: TestCase = TestCase {
        pos: ivec2(3, 3),
        dir: AbsDir::W,
        range: 5,
        opts: 0,
        expected_map: indoc! {"
            . . . . .
            . . . . .
            . . . @ =
            . . . . .
            . . . . .
        "},
        expected_ids: &[],
        expected_dirs: &[],
        expected_cooldown: BotRadar::COOLDOWN_5X5
            + BotRadar::COOLDOWN_OPT_TILES
            + BotRadar::COOLDOWN_OPT_BOTS
            + BotRadar::COOLDOWN_OPT_OBJS,
    };

    const TEST_5X5_S: TestCase = TestCase {
        pos: ivec2(3, 3),
        dir: AbsDir::S,
        range: 5,
        opts: 0,
        expected_map: indoc! {"
            . . . . .
            . . . . .
            . . . . .
            . . @ . .
            . . = . .
        "},
        expected_ids: &[],
        expected_dirs: &[],
        expected_cooldown: BotRadar::COOLDOWN_5X5
            + BotRadar::COOLDOWN_OPT_TILES
            + BotRadar::COOLDOWN_OPT_BOTS
            + BotRadar::COOLDOWN_OPT_OBJS,
    };

    const TEST_OPTS_TILES: TestCase = TestCase {
        pos: ivec2(3, 2),
        dir: AbsDir::N,
        range: 3,
        opts: api::RADAR_SCAN_TILES,
        expected_map: indoc! {"
            . . .
            . . .
            . . .
        "},
        expected_ids: &[],
        expected_dirs: &[],
        expected_cooldown: BotRadar::COOLDOWN_3X3
            + BotRadar::COOLDOWN_OPT_TILES,
    };

    const TEST_OPTS_BOTS: TestCase = TestCase {
        pos: ivec2(3, 2),
        dir: AbsDir::N,
        range: 3,
        opts: api::RADAR_SCAN_BOTS,
        expected_map: indoc! {"
            \0 \0 \0
            \0 @ \0
            \0 \0 \0
        "},
        expected_ids: &[],
        expected_dirs: &[],
        expected_cooldown: BotRadar::COOLDOWN_3X3 + BotRadar::COOLDOWN_OPT_BOTS,
    };

    const TEST_OPTS_BOTS_AND_IDS: TestCase = TestCase {
        pos: ivec2(3, 2),
        dir: AbsDir::N,
        range: 3,
        opts: api::RADAR_SCAN_BOTS | api::RADAR_SCAN_IDS,
        expected_map: indoc! {"
            \0 \0 \0
            \0 @ \0
            \0 \0 \0
        "},
        expected_ids: &[(ivec2(0, 0), Id::new(0xcafed00d))],
        expected_dirs: &[],
        expected_cooldown: BotRadar::COOLDOWN_3X3
            + BotRadar::COOLDOWN_OPT_BOTS
            + BotRadar::COOLDOWN_OPT_IDS,
    };

    const TEST_OPTS_BOTS_AND_DIRS: TestCase = TestCase {
        pos: ivec2(3, 2),
        dir: AbsDir::N,
        range: 3,
        opts: api::RADAR_SCAN_BOTS | api::RADAR_SCAN_DIRS,
        expected_map: indoc! {"
            \0 \0 \0
            \0 @ \0
            \0 \0 \0
        "},
        expected_ids: &[],
        expected_dirs: &[(ivec2(0, 0), '>')],
        expected_cooldown: BotRadar::COOLDOWN_3X3
            + BotRadar::COOLDOWN_OPT_BOTS
            + BotRadar::COOLDOWN_OPT_DIRS,
    };

    const TEST_OPTS_OBJS: TestCase = TestCase {
        pos: ivec2(3, 2),
        dir: AbsDir::N,
        range: 3,
        opts: api::RADAR_SCAN_OBJS,
        expected_map: indoc! {"
            \0 = \0
            \0 \0 \0
            \0 \0 \0
        "},
        expected_ids: &[],
        expected_dirs: &[],
        expected_cooldown: BotRadar::COOLDOWN_3X3 + BotRadar::COOLDOWN_OPT_OBJS,
    };

    const TEST_OPTS_OBJS_AND_IDS: TestCase = TestCase {
        pos: ivec2(3, 2),
        dir: AbsDir::N,
        range: 3,
        opts: api::RADAR_SCAN_OBJS | api::RADAR_SCAN_IDS,
        expected_map: indoc! {"
            \0 = \0
            \0 \0 \0
            \0 \0 \0
        "},
        expected_ids: &[(ivec2(0, -1), Id::new(0xcafebabe))],
        expected_dirs: &[],
        expected_cooldown: BotRadar::COOLDOWN_3X3
            + BotRadar::COOLDOWN_OPT_OBJS
            + BotRadar::COOLDOWN_OPT_IDS,
    };

    const TEST_OPTS_BOTS_AND_OBJS: TestCase = TestCase {
        pos: ivec2(3, 2),
        dir: AbsDir::N,
        range: 3,
        opts: api::RADAR_SCAN_BOTS | api::RADAR_SCAN_OBJS,
        expected_map: indoc! {"
            \0 = \0
            \0 @ \0
            \0 \0 \0
        "},
        expected_ids: &[],
        expected_dirs: &[],
        expected_cooldown: BotRadar::COOLDOWN_3X3
            + BotRadar::COOLDOWN_OPT_BOTS
            + BotRadar::COOLDOWN_OPT_OBJS,
    };

    /// Scanning just the ids is pretty much nonsensical, because it doesn't
    /// return any information (you have to specify `| bots` or `| objs` as
    /// well), but it's legal.
    const TEST_OPTS_IDS: TestCase = TestCase {
        pos: ivec2(3, 2),
        dir: AbsDir::N,
        range: 3,
        opts: api::RADAR_SCAN_IDS,
        expected_map: indoc! {"
            \0 \0 \0
            \0 \0 \0
            \0 \0 \0
        "},
        expected_ids: &[],
        expected_dirs: &[],
        expected_cooldown: BotRadar::COOLDOWN_3X3 + BotRadar::COOLDOWN_OPT_IDS,
    };

    /// Scanning just the dirs is pretty much nonsensical, because it doesn't
    /// return any information (you have to specify `| bots` as well), but it's
    /// legal.
    const TEST_OPTS_DIRS: TestCase = TestCase {
        pos: ivec2(3, 2),
        dir: AbsDir::N,
        range: 3,
        opts: api::RADAR_SCAN_DIRS,
        expected_map: indoc! {"
            \0 \0 \0
            \0 \0 \0
            \0 \0 \0
        "},
        expected_ids: &[],
        expected_dirs: &[],
        expected_cooldown: BotRadar::COOLDOWN_3X3 + BotRadar::COOLDOWN_OPT_DIRS,
    };

    #[test_case(TEST_3X3_1)]
    #[test_case(TEST_3X3_2)]
    #[test_case(TEST_3X3_3)]
    #[test_case(TEST_3X3_4)]
    #[test_case(TEST_5X5_N)]
    #[test_case(TEST_5X5_E)]
    #[test_case(TEST_5X5_W)]
    #[test_case(TEST_5X5_S)]
    #[test_case(TEST_OPTS_TILES)]
    #[test_case(TEST_OPTS_BOTS)]
    #[test_case(TEST_OPTS_BOTS_AND_IDS)]
    #[test_case(TEST_OPTS_BOTS_AND_DIRS)]
    #[test_case(TEST_OPTS_OBJS)]
    #[test_case(TEST_OPTS_OBJS_AND_IDS)]
    #[test_case(TEST_OPTS_BOTS_AND_OBJS)]
    #[test_case(TEST_OPTS_IDS)]
    #[test_case(TEST_OPTS_DIRS)]
    fn test(case: TestCase) {
        let mut world = World::default();

        world.bots.alive.add(Box::new(AliveBot {
            body: AliveBotBody {
                id: BotId::new(0xcafed00d),
                pos: ivec2(3, 2),
                dir: AbsDir::E,
                ..Default::default()
            },
            ..Default::default()
        }));

        world.map = Map::new(uvec2(7, 7));
        world.map.rect(ivec2(0, 0), ivec2(6, 6), TileKind::FLOOR);

        world.objects.add(
            ObjectId::new(0xcafebabe),
            Object::new(ObjectKind::FLAG),
            Some(ivec2(3, 1)),
        );

        // ---

        let mut bot = AliveBotBody {
            pos: case.pos,
            dir: case.dir,
            ..Default::default()
        };

        for addr in [0x00, 0x01] {
            bot.radar.cooldown = 0;

            BotRadar::store(
                &mut bot,
                &mut world,
                api::RADAR_MEM,
                u32::from_le_bytes([0x01, case.range, case.opts, addr]),
            )
            .unwrap();

            let addr = BotRadarAddr::new(addr).unwrap();
            let range = BotRadarRange::new(case.range).unwrap();

            dbg!(addr);
            dbg!(range);

            assert_eq!(
                case.expected_map.trim(),
                BotRadar::read_map(&bot, range, addr).trim()
            );
            assert_eq!(
                case.expected_ids,
                BotRadar::read_ids(&bot, range, addr).collect::<Vec<_>>()
            );
            assert_eq!(
                case.expected_dirs,
                BotRadar::read_dirs(&bot, range, addr).collect::<Vec<_>>()
            );
            assert_eq!(
                [api::IRQ_RADAR_BUSY, 0x00, 0x00, 0x00],
                bot.irq.take().unwrap().to_le_bytes(),
            );
            assert_eq!(case.expected_cooldown, bot.radar.cooldown);
        }

        // ---

        bot.radar.cooldown = 1;

        BotRadar::tick(&mut bot);

        assert_eq!(
            [api::IRQ_RADAR_IDLE, 0x00, 0x00, 0x00],
            bot.irq.take().unwrap().to_le_bytes(),
        );
    }

    impl BotRadar {
        fn read_map(
            bot: &AliveBotBody,
            range: BotRadarRange,
            addr: BotRadarAddr,
        ) -> String {
            range
                .iter_1d()
                .map(|y| {
                    range
                        .iter_1d()
                        .map(|x| {
                            BotRadar::load(bot, addr.get(range, ivec3(x, y, 0)))
                                .unwrap()
                        })
                        .map(|ch| ch as u8 as char)
                        .join(" ")
                })
                .join("\n")
        }

        fn read_ids(
            bot: &AliveBotBody,
            range: BotRadarRange,
            addr: BotRadarAddr,
        ) -> impl Iterator<Item = (IVec2, Id)> + '_ {
            range.iter_2d().filter_map(move |off| {
                let hi = addr.get(range, off.extend(1));
                let lo = addr.get(range, off.extend(2));

                let hi = BotRadar::load(bot, hi).unwrap() as u64;
                let lo = BotRadar::load(bot, lo).unwrap() as u64;

                let id = Id::try_new((hi << 32) | lo)?;

                Some((off, id))
            })
        }

        fn read_dirs(
            bot: &AliveBotBody,
            range: BotRadarRange,
            addr: BotRadarAddr,
        ) -> impl Iterator<Item = (IVec2, char)> + '_ {
            range.iter_2d().filter_map(move |off| {
                let dir = BotRadar::load(bot, addr.get(range, off.extend(0)))
                    .unwrap()
                    .to_le_bytes()[1];

                if dir == 0 {
                    None
                } else {
                    Some((off, dir as char))
                }
            })
        }
    }

    impl BotRadarAddr {
        fn get(&self, range: BotRadarRange, off: IVec3) -> u32 {
            api::RADAR_MEM + 4 * (self.idx(range, off) + 1) as u32
        }
    }
}
