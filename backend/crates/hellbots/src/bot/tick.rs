use crate::{BotId, Bots, Map, Mode};
use anyhow::{Context, Result};
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
    ) -> Result<()> {
        if let Some(dir) = self.stab_dir {
            if let Some(killed_id) = bots.alive.pos_to_id(pos + dir) {
                bots.kill(
                    rng,
                    mode,
                    map,
                    killed_id,
                    "stabbed out of existence",
                    Some(id),
                )
                .context("kill() failed")?;
            }
        }

        if let Some(dir) = self.move_dir {
            let pos = pos + dir;
            let tile = map.get(pos);

            if tile.is_void() {
                bots.kill(rng, mode, map, id, "fell into the void", None)
                    .context("kill() failed")?;
            } else if tile.is_floor() {
                bots.alive.relocate(id, pos);
            }
        }

        Ok(())
    }
}
