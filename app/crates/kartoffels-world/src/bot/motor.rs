use super::AliveBotBody;
use crate::{Event, RelDir, TileKind, World};
use kartoffel as api;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BotMotor {
    cooldown: u32,
}

impl BotMotor {
    pub(super) fn tick(bot: &mut AliveBotBody) {
        if bot.motor.cooldown == 1 {
            bot.irq.raise(api::IRQ_MOTOR_IDLE, [0x00, 0x00, 0x00]);
        }

        bot.motor.cooldown = bot.motor.cooldown.saturating_sub(1);
    }

    pub(super) fn load(bot: &AliveBotBody, addr: u32) -> Result<u32, ()> {
        match addr {
            api::MOTOR_MEM => Ok((bot.motor.cooldown == 0) as u32),
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
            (api::MOTOR_MEM, [0x01, 0x01, 0x01, 0x00]) => {
                Self::do_move(bot, world, RelDir::Up);
                Ok(())
            }
            (api::MOTOR_MEM, [0x01, 0xff, 0xff, 0x00]) => {
                Self::do_move(bot, world, RelDir::Down);
                Ok(())
            }
            (api::MOTOR_MEM, [0x01, 0x01, 0xff, 0x00]) => {
                Self::do_move(bot, world, RelDir::Right);
                Ok(())
            }
            (api::MOTOR_MEM, [0x01, 0xff, 0x01, 0x00]) => {
                Self::do_move(bot, world, RelDir::Left);
                Ok(())
            }

            _ => Err(()),
        }
    }

    fn do_move(bot: &mut AliveBotBody, world: &mut World, dir: RelDir) {
        if bot.motor.cooldown > 0 {
            return;
        }

        let ok;

        match dir {
            RelDir::Up | RelDir::Down => {
                let at = bot.pos + dir * bot.dir;

                match world.map.get(at).kind {
                    TileKind::VOID => {
                        ok = true;
                        bot.pos = AliveBotBody::FELL_INTO_VOID;
                    }

                    TileKind::FLOOR => {
                        if world.bots.alive.lookup_at(at).is_none()
                            && world.objects.lookup_at(at).is_none()
                        {
                            ok = true;
                            bot.pos = at;

                            world
                                .events
                                .add(Event::BotMoved { id: bot.id, at });
                        } else {
                            ok = false;

                            bot.events.add(
                                &world.clock,
                                format!(
                                    "couldn't move at {},{}: tile occupied",
                                    at.x, at.y
                                ),
                            );
                        }
                    }

                    _ => {
                        ok = false;

                        bot.events.add(
                            &world.clock,
                            format!(
                                "couldn't move at {},{}: tile occupied",
                                at.x, at.y
                            ),
                        );
                    }
                }
            }

            RelDir::Left | RelDir::Right => {
                ok = true;
                bot.dir = dir * bot.dir;
            }
        }

        bot.irq.raise(api::IRQ_MOTOR_BUSY, {
            let a0 = dir.as_caret() as u8;

            let a1 = if ok {
                api::MOTOR_STAT_OK
            } else {
                api::MOTOR_STAT_ERR
            };

            let a2 = if ok {
                let at = bot.pos + bot.dir;

                if world.bots.alive.lookup_at(at).is_some() {
                    b'@'
                } else if let Some((_, obj)) = world.objects.get_at(at) {
                    obj.kind
                } else {
                    world.map.get(at).kind
                }
            } else {
                api::MOTOR_ERR_BLOCKED
            };

            [a0, a1, a2]
        });

        bot.motor.cooldown = world.cooldown(match dir {
            RelDir::Up => 20_000,
            RelDir::Down => 30_000,
            _ => 25_000,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AbsDir, AliveBot, Map, Object, ObjectId, ObjectKind};
    use glam::{IVec2, ivec2, uvec2};
    use test_case::test_case;

    #[derive(Clone, Debug)]
    struct TestCase {
        cmd: u32,
        pos: IVec2,
        expected_pos: IVec2,
        expected_dir: AbsDir,
        expected_irq: [u8; 4],
        expected_cooldown: u32,
    }

    const TEST_MOVE_FW: TestCase = TestCase {
        cmd: api::pack(0x01, 0x01, 0x01, 0x00),
        pos: ivec2(1, 1),
        expected_pos: ivec2(1, 0),
        expected_dir: AbsDir::N,
        expected_irq: [api::IRQ_MOTOR_BUSY, b'^', api::MOTOR_STAT_OK, b' '],
        expected_cooldown: 20_000,
    };

    const TEST_MOVE_BW: TestCase = TestCase {
        cmd: api::pack(0x01, 0xff, 0xff, 0x00),
        pos: ivec2(1, 1),
        expected_pos: ivec2(1, 2),
        expected_dir: AbsDir::N,
        expected_irq: [api::IRQ_MOTOR_BUSY, b'v', api::MOTOR_STAT_OK, b'.'],
        expected_cooldown: 30_000,
    };

    const TEST_TURN_LEFT: TestCase = TestCase {
        cmd: api::pack(0x01, 0xff, 0x01, 0x00),
        pos: ivec2(1, 1),
        expected_pos: ivec2(1, 1),
        expected_dir: AbsDir::W,
        expected_irq: [api::IRQ_MOTOR_BUSY, b'<', api::MOTOR_STAT_OK, b'.'],
        expected_cooldown: 25_000,
    };

    const TEST_TURN_RIGHT: TestCase = TestCase {
        cmd: api::pack(0x01, 0x01, 0xff, 0x00),
        pos: ivec2(1, 1),
        expected_pos: ivec2(1, 1),
        expected_dir: AbsDir::E,
        expected_irq: [api::IRQ_MOTOR_BUSY, b'>', api::MOTOR_STAT_OK, b'*'],
        expected_cooldown: 25_000,
    };

    const TEST_BLOCKED: TestCase = TestCase {
        cmd: api::pack(0x01, 0x01, 0x01, 0x00),
        pos: ivec2(2, 1),
        expected_pos: ivec2(2, 1),
        expected_dir: AbsDir::N,
        expected_irq: [
            api::IRQ_MOTOR_BUSY,
            b'^',
            api::MOTOR_STAT_ERR,
            api::MOTOR_ERR_BLOCKED,
        ],
        expected_cooldown: 20_000,
    };

    #[test_case(TEST_MOVE_FW)]
    #[test_case(TEST_MOVE_BW)]
    #[test_case(TEST_TURN_LEFT)]
    #[test_case(TEST_TURN_RIGHT)]
    #[test_case(TEST_BLOCKED)]
    fn smoke(case: TestCase) {
        let mut bot = AliveBot::default();
        let mut world = World::default();

        bot.pos = case.pos;
        bot.dir = AbsDir::N;

        world.map = Map::new(uvec2(3, 3));
        world.map.fill(TileKind::FLOOR);
        world.map.set(ivec2(2, 0), TileKind::WALL);

        world.objects.add(
            ObjectId::new(321),
            Object::new(ObjectKind::GEM),
            ivec2(2, 1),
        );

        // ---

        bot.store(&mut world, api::MOTOR_MEM, case.cmd).unwrap();

        assert_eq!(case.expected_pos, bot.pos);
        assert_eq!(case.expected_dir, bot.dir);
        assert_eq!(case.expected_irq, bot.irq.take_le().unwrap());
        assert_eq!(case.expected_cooldown, bot.motor.cooldown);

        // ---
        // Make sure the second command is no-op (cooldown > 0)

        bot.store(&mut world, api::MOTOR_MEM, case.cmd).unwrap();

        assert_eq!(bot.pos, case.expected_pos);
        assert_eq!(bot.dir, case.expected_dir);
        assert_eq!(None, bot.irq.take());
        assert_eq!(bot.motor.cooldown, case.expected_cooldown);

        // ---
        // Make sure we emit the "motor idle" IRQ once the cooldown cools down

        bot.motor.cooldown = 2;
        BotMotor::tick(&mut bot);

        assert_eq!(None, bot.irq.take());

        bot.motor.cooldown = 1;
        BotMotor::tick(&mut bot);

        assert_eq!(
            Some([api::IRQ_MOTOR_IDLE, 0x00, 0x00, 0x00]),
            bot.irq.take_le(),
        );
    }
}
