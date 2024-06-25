use crate::{BotId, Bots, Map, Mode};
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
        mode: &mut Mode,
        map: &mut Map,
        bots: &mut Bots,
        id: BotId,
        pos: IVec2,
    ) {
        if let Some(dir) = self.stab_dir {
            if let Some(killed_id) = bots.alive.lookup_by_pos(pos + dir) {
                bots.kill(
                    rng,
                    mode,
                    killed_id,
                    "stabbed out of existence",
                    Some(id),
                );
            }
        }

        if let Some(dir) = self.move_dir {
            let pos = pos + dir;
            let tile = map.get(pos);

            if tile.is_void() {
                bots.kill(rng, mode, id, "fell into the void", None);
            } else if tile.is_floor() && bots.alive.lookup_by_pos(pos).is_none()
            {
                bots.alive.relocate(id, pos);
            }
        }
    }
}
