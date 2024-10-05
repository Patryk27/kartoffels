use crate::{BotId, Dir, KillBot, World};
use glam::IVec2;

#[derive(Debug)]
pub struct AliveBotTick {
    pub pos: IVec2,
    pub stab_dir: Option<Dir>,
    pub move_dir: Option<Dir>,
}

impl AliveBotTick {
    pub fn apply(self, world: &mut World, id: BotId) -> Option<KillBot> {
        if let Some(dir) = self.stab_dir {
            if let Some(killed_id) = world.bots.alive.get_by_pos(self.pos + dir)
            {
                return Some(KillBot {
                    id: killed_id,
                    reason: format!("stabbed out of existence by {}", id),
                    killer: Some(id),
                });
            }
        }

        if let Some(dir) = self.move_dir {
            let pos = self.pos + dir;
            let tile = world.map.get(pos);

            if tile.is_void() {
                return Some(KillBot {
                    id,
                    reason: "fell into the void".into(),
                    killer: None,
                });
            }

            if tile.is_floor() && world.bots.alive.get_by_pos(pos).is_none() {
                world.bots.alive.relocate(id, pos);
            }
        }

        None
    }
}
