mod alive;
mod dead;
mod queued;

pub use self::alive::*;
pub use self::dead::*;
pub use self::queued::*;
use crate::{AliveBot, BotId, DeadBot, Map, Mode, Policy, QueuedBot};
use anyhow::{anyhow, Result};
use chrono::Utc;
use glam::IVec2;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::debug;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Bots {
    pub alive: AliveBots,
    pub dead: DeadBots,
    pub queued: QueuedBots,
}

impl Bots {
    pub fn add(
        &mut self,
        rng: &mut impl RngCore,
        policy: &Policy,
        bot: AliveBot,
    ) -> Result<BotId> {
        let id = self.random_unoccupied_id(rng);

        if self.queued.len() < policy.max_queued_bots {
            self.queued.push(QueuedBot {
                id,
                bot,
                requeued: false,
            });

            debug!(?id, "bot queued");

            Ok(id)
        } else {
            debug!(?id, "bot discarded (queue full)");

            Err(anyhow!("too many robots queued, try again in a moment"))
        }
    }

    pub fn kill(
        &mut self,
        rng: &mut impl RngCore,
        mode: &mut Mode,
        policy: &Policy,
        id: BotId,
        reason: &str,
        killer: Option<BotId>,
    ) {
        debug!(?id, ?reason, ?killer, "bot killed");

        let bot = self.alive.remove(id);

        self.dead.add(
            id,
            DeadBot {
                reason: Arc::new(reason.into()),
                killed_at: Utc::now(),
            },
        );

        mode.on_bot_killed(id, killer);

        if let Some(bot) = bot.reset(rng) {
            if self.queued.len() < policy.max_queued_bots {
                debug!(?id, "bot requeued");

                self.queued.push(QueuedBot {
                    id,
                    bot,
                    requeued: true,
                });
            } else {
                debug!(?id, "bot discarded (queue is full)");
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
