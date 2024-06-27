use crate::{BotId, World};
use glam::IVec2;
use rand::RngCore;

#[derive(Debug)]
pub struct AliveBotTick {
    pub stab_dir: Option<IVec2>,
    pub move_dir: Option<IVec2>,
}

impl AliveBotTick {
    pub fn apply(
        self,
        rng: &mut impl RngCore,
        world: &mut World,
        id: BotId,
        pos: IVec2,
    ) {
        if let Some(dir) = self.stab_dir {
            if let Some(killed_id) = world.bots.alive.lookup_by_pos(pos + dir) {
                world.bots.kill(
                    rng,
                    &mut world.mode,
                    &world.policy,
                    killed_id,
                    &format!("stabbed out of existence by {}", id),
                    Some(id),
                );
            }
        }

        if let Some(dir) = self.move_dir {
            let pos = pos + dir;
            let tile = world.map.get(pos);

            if tile.is_void() {
                world.bots.kill(
                    rng,
                    &mut world.mode,
                    &world.policy,
                    id,
                    "fell into the void",
                    None,
                );
            } else if tile.is_floor()
                && world.bots.alive.lookup_by_pos(pos).is_none()
            {
                world.bots.alive.relocate(id, pos);
            }
        }
    }
}
