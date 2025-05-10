use super::AliveBotBody;
use crate::{Event, TileKind, World};
use kartoffel as api;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BotArm {
    cooldown: u32,
}

impl BotArm {
    pub(super) fn tick(bot: &mut AliveBotBody) {
        if bot.arm.cooldown == 1 {
            bot.irq.raise(api::IRQ_ARM_IDLE, [0x00, 0x00, 0x00]);
        }

        bot.arm.cooldown = bot.arm.cooldown.saturating_sub(1);
    }

    pub(super) fn load(bot: &AliveBotBody, addr: u32) -> Result<u32, ()> {
        match addr {
            api::ARM_MEM => Ok((bot.arm.cooldown == 0) as u32),
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
            (api::ARM_MEM, [api::ARM_CMD_STAB, 0x00, 0x00, 0x00]) => {
                Self::do_stab(bot, world);
                Ok(())
            }
            (api::ARM_MEM, [api::ARM_CMD_PICK, 0x00, 0x00, 0x00]) => {
                Self::do_pick(bot, world);
                Ok(())
            }
            (api::ARM_MEM, [api::ARM_CMD_DROP, idx, 0x00, 0x00]) => {
                Self::do_drop(bot, world, idx);
                Ok(())
            }

            _ => Err(()),
        }
    }

    fn do_stab(bot: &mut AliveBotBody, world: &mut World) {
        if bot.arm.cooldown > 0 {
            return;
        }

        let at = bot.pos + bot.dir;

        let Some(killed) = world.bots.alive.remove_at(at) else {
            bot.irq.raise(
                api::IRQ_ARM_BUSY,
                [
                    api::ARM_CMD_STAB,
                    api::ARM_STAT_ERR,
                    api::ARM_ERR_NOBODY_THERE,
                ],
            );

            bot.events.add(
                &world.clock,
                format!("couldn't stab at {},{}: nobody there", at.x, at.y),
            );

            bot.arm.cooldown = world.cooldown(45_000);
            return;
        };

        bot.irq.raise(
            api::IRQ_ARM_BUSY,
            [api::ARM_CMD_STAB, api::ARM_STAT_OK, 0x00],
        );

        bot.events
            .add(&world.clock, format!("killed {} (knife)", killed.id));

        world.kill_bot(
            killed,
            format!("killed by {} (knife)", bot.id),
            Some(bot.id),
        );

        bot.arm.cooldown = world.cooldown(60_000);
    }

    fn do_pick(bot: &mut AliveBotBody, world: &mut World) {
        if bot.arm.cooldown > 0 {
            return;
        }

        let at = bot.pos + bot.dir;

        let Some((id, obj)) = world.objects.remove_at(at) else {
            bot.irq.raise(
                api::IRQ_ARM_BUSY,
                [
                    api::ARM_CMD_PICK,
                    api::ARM_STAT_ERR,
                    api::ARM_ERR_NOTHING_THERE,
                ],
            );

            bot.events.add(
                &world.clock,
                format!("couldn't pick from {},{}: nothing there", at.x, at.y),
            );

            bot.arm.cooldown = world.cooldown(45_000);
            return;
        };

        let Ok(idx) = bot.inventory.add(id, obj) else {
            bot.irq.raise(
                api::IRQ_ARM_BUSY,
                [
                    api::ARM_CMD_PICK,
                    api::ARM_STAT_ERR,
                    api::ARM_ERR_INVENTORY_FULL,
                ],
            );

            bot.events.add(
                &world.clock,
                format!(
                    "couldn't pick {} from {},{}: inventory full",
                    obj.name(),
                    at.x,
                    at.y
                ),
            );

            world.objects.add(id, obj, at);
            bot.arm.cooldown = world.cooldown(45_000);
            return;
        };

        bot.irq
            .raise(api::IRQ_ARM_BUSY, [api::ARM_CMD_PICK, obj.kind, idx]);

        bot.events.add(
            &world.clock,
            format!("picked {} from {},{}", obj.name(), at.x, at.y),
        );

        world.events.add(Event::ObjectPicked { id });
        bot.arm.cooldown = world.cooldown(60_000);
    }

    fn do_drop(bot: &mut AliveBotBody, world: &mut World, idx: u8) {
        if bot.arm.cooldown > 0 {
            return;
        }

        let at = bot.pos + bot.dir;

        if world.map.get(at).kind != TileKind::FLOOR
            || world.objects.get_at(at).is_some()
        {
            bot.irq.raise(
                api::IRQ_ARM_BUSY,
                [
                    api::ARM_CMD_DROP,
                    api::ARM_STAT_ERR,
                    api::ARM_ERR_NEEDS_FLOOR,
                ],
            );

            bot.events.add(
                &world.clock,
                format!(
                    "couldn't drop #{idx} at {},{}: need a floor",
                    at.x, at.y
                ),
            );

            bot.arm.cooldown = world.cooldown(45_000);
            return;
        }

        let Some((id, obj)) = bot.inventory.take(idx) else {
            bot.irq.raise(
                api::IRQ_ARM_BUSY,
                [
                    api::ARM_CMD_DROP,
                    api::ARM_STAT_ERR,
                    api::ARM_ERR_NO_SUCH_OBJECT,
                ],
            );

            bot.events.add(
                &world.clock,
                format!(
                    "couldn't drop #{idx} at {},{}: no such object in \
                     inventory",
                    at.x, at.y
                ),
            );

            bot.arm.cooldown = world.cooldown(45_000);
            return;
        };

        bot.irq
            .raise(api::IRQ_ARM_BUSY, [api::ARM_CMD_DROP, obj.kind, idx]);

        bot.events.add(
            &world.clock,
            format!("dropped #{idx} ({}) at {},{}", obj.name(), at.x, at.y),
        );

        world.events.add(Event::ObjectDropped { id });
        world.objects.add(id, obj, at);
        bot.arm.cooldown = world.cooldown(60_000);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AbsDir, AliveBot, BotId, Map, Object, ObjectId, ObjectKind};
    use glam::{ivec2, uvec2};

    #[test]
    fn stab() {
        let mut bot = AliveBot::default();
        let mut world = World::default();

        bot.id = BotId::new(1234);
        bot.pos = ivec2(0, 0);
        bot.dir = AbsDir::E;

        world.bots.alive.add(Box::new(AliveBot {
            body: AliveBotBody {
                id: BotId::new(4321),
                pos: ivec2(1, 0),
                ..Default::default()
            },
            ..Default::default()
        }));

        world.lives.on_bot_born(&world.clock, BotId::new(1234));
        world.lives.on_bot_born(&world.clock, BotId::new(4321));

        // ---

        bot.store(
            &mut world,
            api::ARM_MEM,
            api::pack(api::ARM_CMD_STAB, 0x00, 0x00, 0x00),
        )
        .unwrap();

        assert_eq!(60_000, bot.arm.cooldown);

        assert_eq!(
            Some("killed 0000-0000-0000-10e1 (knife)"),
            bot.events.newest(),
        );

        assert!(world.bots.dead.contains(BotId::new(4321)));

        assert_eq!(
            [api::IRQ_ARM_BUSY, api::ARM_CMD_STAB, api::ARM_STAT_OK, 0x00],
            bot.irq.take().unwrap().to_le_bytes(),
        );

        // ---

        bot.arm.cooldown = 0;

        bot.store(
            &mut world,
            api::ARM_MEM,
            api::pack(api::ARM_CMD_STAB, 0x00, 0x00, 0x00),
        )
        .unwrap();

        assert_eq!(45_000, bot.arm.cooldown);

        assert_eq!(
            Some("couldn't stab at 1,0: nobody there"),
            bot.events.newest(),
        );

        assert_eq!(
            Some([
                api::IRQ_ARM_BUSY,
                api::ARM_CMD_STAB,
                api::ARM_STAT_ERR,
                api::ARM_ERR_NOBODY_THERE,
            ]),
            bot.irq.take_le(),
        );
    }

    #[test]
    fn pick_and_drop() {
        let mut bot = AliveBot::default();
        let mut world = World::default();

        bot.pos = ivec2(0, 0);
        bot.dir = AbsDir::E;

        world.map = Map::new(uvec2(3, 3));
        world.map.fill(TileKind::FLOOR);

        // ---

        for idx in [0x00, 0x01] {
            world.objects.add(
                ObjectId::new(123),
                Object::new(ObjectKind::GEM),
                ivec2(1, 0),
            );

            bot.arm.cooldown = 0;

            bot.store(
                &mut world,
                api::ARM_MEM,
                api::pack(api::ARM_CMD_PICK, 0x00, 0x00, 0x00),
            )
            .unwrap();

            assert_eq!(60_000, bot.arm.cooldown);
            assert_eq!(Some("picked gem from 1,0"), bot.events.newest());

            assert_eq!(
                Some([
                    api::IRQ_ARM_BUSY,
                    api::ARM_CMD_PICK,
                    ObjectKind::GEM,
                    idx,
                ]),
                bot.irq.take_le()
            );
        }

        // ---

        bot.arm.cooldown = 0;

        bot.store(
            &mut world,
            api::ARM_MEM,
            api::pack(api::ARM_CMD_PICK, 0x00, 0x00, 0x00),
        )
        .unwrap();

        assert_eq!(45_000, bot.arm.cooldown);

        assert_eq!(
            Some("couldn't pick from 1,0: nothing there"),
            bot.events.newest(),
        );

        assert_eq!(
            Some([
                api::IRQ_ARM_BUSY,
                api::ARM_CMD_PICK,
                api::ARM_STAT_ERR,
                api::ARM_ERR_NOTHING_THERE,
            ]),
            bot.irq.take_le(),
        );

        // ---

        for _ in 0..2 {
            bot.arm.cooldown = 0;

            bot.store(
                &mut world,
                api::ARM_MEM,
                api::pack(api::ARM_CMD_DROP, 0x00, 0x00, 0x00),
            )
            .unwrap();

            assert_eq!(60_000, bot.arm.cooldown);
            assert_eq!(Some("dropped #0 (gem) at 1,0"), bot.events.newest());

            assert_eq!(
                Some([
                    api::IRQ_ARM_BUSY,
                    api::ARM_CMD_DROP,
                    ObjectKind::GEM,
                    0x00,
                ]),
                bot.irq.take_le(),
            );

            assert_eq!(
                ObjectKind::GEM,
                world.objects.remove_at(ivec2(1, 0)).unwrap().1.kind,
            );
        }

        // ---

        bot.arm.cooldown = 0;

        bot.store(
            &mut world,
            api::ARM_MEM,
            api::pack(api::ARM_CMD_DROP, 0x00, 0x00, 0x00),
        )
        .unwrap();

        assert_eq!(45_000, bot.arm.cooldown);

        assert_eq!(
            Some("couldn't drop #0 at 1,0: no such object in inventory"),
            bot.events.newest(),
        );

        assert_eq!(
            Some([
                api::IRQ_ARM_BUSY,
                api::ARM_CMD_DROP,
                api::ARM_STAT_ERR,
                api::ARM_ERR_NO_SUCH_OBJECT,
            ]),
            bot.irq.take_le(),
        );

        // ---

        world.map.fill(TileKind::VOID);

        bot.arm.cooldown = 0;

        bot.store(
            &mut world,
            api::ARM_MEM,
            api::pack(api::ARM_CMD_DROP, 0x00, 0x00, 0x00),
        )
        .unwrap();

        assert_eq!(45_000, bot.arm.cooldown);

        assert_eq!(
            Some("couldn't drop #0 at 1,0: need a floor"),
            bot.events.newest(),
        );

        assert_eq!(
            Some([
                api::IRQ_ARM_BUSY,
                api::ARM_CMD_DROP,
                api::ARM_STAT_ERR,
                api::ARM_ERR_NEEDS_FLOOR
            ]),
            bot.irq.take_le(),
        );
    }
}
