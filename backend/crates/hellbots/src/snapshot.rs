use crate::{BotId, Bots, Map, Mode, Tile};
use anyhow::{Context, Result};
use glam::{ivec2, IVec2};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct WorldSnapshot {
    mode: Value,
    map: Map,

    // TODO sort by upload/restart date
    bots: BTreeMap<BotId, BotSnapshot>,
}

impl WorldSnapshot {
    pub fn mode(&self) -> &Value {
        &self.mode
    }

    pub fn map(&self, min: IVec2, max: IVec2) -> Vec<Tile> {
        let mut out = Vec::with_capacity(16 * 1024);

        for y in min.y..=max.y {
            for x in min.x..=max.x {
                out.push(self.map.get(ivec2(x, y)));
            }
        }

        out
    }

    pub fn center(&self) -> IVec2 {
        self.map.center()
    }

    pub fn bot(&self, id: BotId) -> Option<&BotSnapshot> {
        self.bots.get(&id)
    }

    pub fn bots(&self) -> impl Iterator<Item = (BotId, &BotSnapshot)> {
        self.bots.iter().map(|(id, bot)| (*id, bot))
    }

    pub(super) fn update(
        &mut self,
        mode: &Mode,
        map: &Map,
        bots: &Bots,
    ) -> Result<()> {
        self.mode = mode.state().context("state() failed")?;
        self.map = map.clone();
        self.bots = bots.alive.snapshots().collect();

        for (bot_idx, bot) in self.bots.values().enumerate() {
            self.map.set(bot.pos, Tile::bot(bot_idx as u8));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct BotSnapshot {
    pub pos: IVec2,
    pub uart: String,
}
