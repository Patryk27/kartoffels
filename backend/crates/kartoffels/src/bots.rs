mod alive;
mod dead;
mod queued;

pub use self::alive::*;
pub use self::dead::*;
pub use self::queued::*;
use crate::{AliveBot, BotId, DeadBot, Map, Mode, QueuedBot};
use anyhow::{anyhow, Context, Result};
use glam::IVec2;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Bots {
    pub alive: AliveBots,
    pub dead: DeadBots,
    pub queued: QueuedBots,
}

impl Bots {
    pub fn create(
        &mut self,
        rng: &mut impl RngCore,
        map: &Map,
        bot: AliveBot,
    ) -> Result<BotId> {
        let id = self.random_unoccupied_id(rng);

        let pos = self
            .random_unoccupied_pos(map)
            .context("couldn't squeeze any space for the bot")?;

        // TODO make limit configurable
        if self.alive.len() < 64 {
            self.alive.add(id, pos, bot);

            debug!(?id, ?pos, "bot created");
        } else {
            match self.queued.push(QueuedBot { id, bot }) {
                Ok(()) => {
                    debug!(?id, "bot queued");
                }

                Err(()) => {
                    debug!(?id, "bot discarded (queue full)");

                    return Err(anyhow!(
                        "too many robots queued, try again later"
                    ));
                }
            }
        }

        Ok(id)
    }

    pub fn kill(
        &mut self,
        rng: &mut impl RngCore,
        mode: &mut Mode,
        id: BotId,
        reason: &str,
        killer: Option<BotId>,
    ) {
        debug!(?id, ?reason, ?killer, "bot killed");

        let bot = self.alive.remove(id);

        self.dead.add(
            id,
            DeadBot {
                reason: reason.into(),
            },
        );

        mode.on_bot_killed(id, killer);

        if let Some(bot) = bot.reset(rng) {
            match self.queued.push(QueuedBot { id, bot }) {
                Ok(()) => {
                    debug!(?id, "bot requeued");
                }

                Err(()) => {
                    debug!(?id, "bot discarded (queue is full)");
                }
            }
        } else {
            debug!(?id, "bot discarded (couldn't be reset)");
        }
    }

    pub fn has(&self, id: BotId) -> bool {
        self.alive.has(id) || self.dead.has(id) || self.queued.has(id)
    }

    pub fn random_unoccupied_id(&self, rng: &mut impl RngCore) -> BotId {
        loop {
            let id = BotId::new(rng);

            if !self.has(id) {
                break id;
            }
        }
    }

    pub fn random_unoccupied_pos(&self, map: &Map) -> Option<IVec2> {
        let mut rng = rand::thread_rng();
        let mut nth = 0;

        loop {
            let pos = map.rand_pos(&mut rng);

            if map.get(pos).is_floor()
                && self.alive.lookup_by_pos(pos).is_none()
            {
                return Some(pos);
            }

            nth += 1;

            if nth >= 1024 {
                return None;
            }
        }
    }
}
