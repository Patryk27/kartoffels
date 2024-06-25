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
        let pos = self.random_unoccupied_pos(map)?;

        if self.alive.remove(id).is_ok() {
            debug!(?id, "bot deleted");
        }

        match self.alive.add(id, pos, bot) {
            Ok(()) => {
                debug!(?id, ?pos, "bot created");
            }

            Err(bot) => match self.queued.push(QueuedBot { id, bot }) {
                Ok(()) => {
                    debug!(?id, "bot queued");
                }

                Err(()) => {
                    debug!(?id, "bot rejected (queue full)");

                    return Err(anyhow!(
                        "too many robots queued, try again later"
                    ));
                }
            },
        }

        Ok(id)
    }

    pub fn kill(
        &mut self,
        rng: &mut impl RngCore,
        mode: &mut Mode,
        map: &mut Map,
        id: BotId,
        reason: impl ToString,
        killer: Option<BotId>,
    ) -> Result<()> {
        let reason = reason.to_string();
        let bot = self.alive.remove(id)?;

        debug!(?id, ?reason, ?killer, "bot killed");

        self.dead.add(id, DeadBot { reason });

        mode.on_bot_killed(id, killer)
            .context("on_bot_killed() failed")?;

        let pos = self.random_unoccupied_pos(map)?;

        if let Some(QueuedBot { id, bot }) = self.queued.pop() {
            debug!(?id, ?pos, "bot dequeued");

            if self.alive.add(id, pos, bot).is_err() {
                return Err(anyhow!("couldn't spawn dequeued bot"));
            }
        } else {
            // TODO resurrect after 1s

            if let Some(bot) = bot.reset(rng) {
                debug!(?id, ?pos, "bot resurrected");

                if self.alive.add(id, pos, bot).is_err() {
                    return Err(anyhow!("couldn't spawn resurrected bot"));
                }
            } else {
                // TODO
            }
        }

        Ok(())
    }

    pub fn has(&self, id: BotId) -> bool {
        self.alive.has(id) || self.dead.has(id) || self.queued.has(id)
    }

    fn random_unoccupied_id(&self, rng: &mut impl RngCore) -> BotId {
        loop {
            let id = BotId::new(rng);

            if !self.has(id) {
                break id;
            }
        }
    }

    fn random_unoccupied_pos(&self, map: &Map) -> Result<IVec2> {
        let mut rng = rand::thread_rng();
        let mut nth = 0;

        loop {
            let pos = map.rand_pos(&mut rng);

            if map.get(pos).is_floor()
                && self.alive.pos_to_id_opt(pos).is_none()
            {
                return Ok(pos);
            }

            nth += 1;

            if nth >= 1024 {
                return Err(anyhow!("couldn't squeeze any space for the bot"));
            }
        }
    }
}
