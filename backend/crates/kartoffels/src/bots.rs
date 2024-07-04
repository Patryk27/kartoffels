mod alive;
mod dead;
mod queued;

pub use self::alive::*;
pub use self::dead::*;
pub use self::queued::*;
use crate::{AliveBot, BotId, DeadBot, Map, Mode, Policy, QueuedBot};
use anyhow::{anyhow, Result};
use glam::IVec2;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Bots {
    pub queued: QueuedBots,
    pub alive: AliveBots,
    pub dead: DeadBots,
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

    pub fn get(&self, id: BotId) -> Option<BotEntry> {
        if let Some(entry) = self.queued.get(id) {
            return Some(BotEntry::Queued(entry));
        }

        if let Some(entry) = self.alive.get(id) {
            return Some(BotEntry::Alive(entry));
        }

        if self.dead.get(id).is_some() {
            return Some(BotEntry::Dead);
        }

        None
    }

    pub fn get_mut(&mut self, id: BotId) -> Option<BotEntryMut> {
        if let Some(entry) = self.queued.get_mut(id) {
            return Some(BotEntryMut::Queued(entry));
        }

        if let Some(entry) = self.alive.get_mut(id) {
            return Some(BotEntryMut::Alive(entry.bot));
        }

        if let Some(entry) = self.dead.get_mut(id) {
            return Some(BotEntryMut::Dead(entry));
        }

        None
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

        mode.on_bot_killed(id, killer);

        let mut bot = self.alive.remove(id);

        bot.log(reason.to_owned());

        if let Some(killer) = killer {
            if let Some(killer) = self.alive.get_mut(killer) {
                killer.bot.log(format!("stabbed {}", id));
            }
        }

        match bot.reset(rng) {
            Ok(mut bot) => {
                if self.queued.len() < policy.max_queued_bots {
                    debug!(?id, "bot requeued");

                    self.queued.push(QueuedBot {
                        id,
                        bot,
                        requeued: true,
                    });
                } else {
                    let msg = "discarded (queue is full)";

                    bot.log(msg.into());
                    debug!(?id, "bot {}", msg);

                    self.dead.add(id, DeadBot { events: bot.events });
                }
            }

            Err(mut bot) => {
                let msg = "discarded (couldn't be reset)";

                bot.log(msg.into());
                debug!(?id, "bot {}", msg);

                self.dead.add(id, DeadBot { events: bot.events });
            }
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

#[derive(Debug)]
pub enum BotEntry<'a> {
    Queued(QueuedBotEntry<'a>),
    Alive(AliveBotEntry<'a>),
    Dead,
}

#[derive(Debug)]
pub enum BotEntryMut<'a> {
    Queued(&'a mut QueuedBot),
    Alive(&'a mut AliveBot),
    Dead(&'a mut DeadBot),
}
