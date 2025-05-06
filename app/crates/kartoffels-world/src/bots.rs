mod alive;
mod dead;
mod queued;
mod systems;

pub use self::alive::*;
pub use self::dead::*;
pub use self::queued::*;
pub use self::systems::*;
use crate::*;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Bots {
    pub alive: AliveBots,
    pub dead: DeadBots,
    pub queued: QueuedBots,
}

impl Bots {
    pub fn create(
        &mut self,
        clock: &Clock,
        events: &mut Events,
        lives: &mut Lives,
        map: &Map,
        objects: &Objects,
        policy: &Policy,
        rng: &mut ChaCha8Rng,
        spawn: &Spawn,
        req: CreateBotRequest,
    ) -> Result<BotId> {
        let CreateBotRequest {
            src,
            pos,
            dir,
            instant,
            oneshot,
        } = req;

        debug!(
            src = ?sha256::digest(&src[..])[0..8],
            ?pos,
            ?dir,
            ?instant,
            ?oneshot,
            "creating bot",
        );

        let id = loop {
            let id = rng.gen();

            if !self.contains(id) {
                break id;
            }
        };

        let bot = {
            let mut events = BotEvents::default();

            if !instant {
                events.add(clock, "uploaded");
            }

            let fw =
                Firmware::from_elf(&src).context("couldn't parse firmware")?;

            Box::new(QueuedBot {
                dir,
                events,
                fw,
                id,
                oneshot,
                pos,
                requeued: false,
                serial: Default::default(),
            })
        };

        if instant {
            self.spawn(clock, events, lives, map, objects, rng, spawn, bot)
                .map_err(|(err, _)| err)?;
        } else {
            if self.queued.len() >= policy.max_queued_bots as usize {
                return Err(anyhow!(
                    "too many bots queued, try again in a moment"
                ));
            }

            self.queued.push_back(bot);
        }

        Ok(id)
    }

    pub fn spawn(
        &mut self,
        clock: &Clock,
        events: &mut Events,
        lives: &mut Lives,
        map: &Map,
        objects: &Objects,
        rng: &mut impl RngCore,
        spawn: &Spawn,
        bot: Box<QueuedBot>,
    ) -> Result<(), (Error, Box<QueuedBot>)> {
        let Some((pos, dir)) =
            self.determine_spawn_point(map, objects, rng, spawn, &bot)
        else {
            return Err((anyhow!("couldn't determine spawn point"), bot));
        };

        let bot = Box::new(AliveBot::new(rng, clock, pos, dir, *bot));
        let id = bot.id;

        trace!(?id, ?pos, ?dir, "spawning bot");

        events.add(Event::BotBorn { id });
        lives.on_bot_born(clock, id);
        self.alive.add(bot);

        Ok(())
    }

    fn determine_spawn_point(
        &self,
        map: &Map,
        objects: &Objects,
        rng: &mut impl RngCore,
        spawn: &Spawn,
        bot: &QueuedBot,
    ) -> Option<(IVec2, Dir)> {
        if let Some(pos) = bot.pos {
            let dir = bot.dir.unwrap_or_else(|| rng.gen());

            return if self.is_pos_legal(map, objects, pos, false) {
                Some((pos, dir))
            } else {
                None
            };
        }

        if let Some(pos) = spawn.pos {
            let dir = spawn.dir.unwrap_or_else(|| rng.gen());

            return if self.is_pos_legal(map, objects, pos, false) {
                Some((pos, dir))
            } else {
                None
            };
        }

        self.sample_map(map, objects, rng, bot)
    }

    fn sample_map(
        &self,
        map: &Map,
        objects: &Objects,
        rng: &mut impl RngCore,
        bot: &QueuedBot,
    ) -> Option<(IVec2, Dir)> {
        if map.size() == UVec2::ZERO {
            return None;
        }

        let mut idx = 0;

        loop {
            let pos = map.sample_pos(rng);

            if self.is_pos_legal(map, objects, pos, true) {
                let dir = bot.dir.unwrap_or_else(|| rng.gen());

                return Some((pos, dir));
            }

            idx += 1;

            if idx >= 1024 {
                return None;
            }
        }
    }

    fn is_pos_legal(
        &self,
        map: &Map,
        objects: &Objects,
        pos: IVec2,
        check_neighborhood: bool,
    ) -> bool {
        if !map.get(pos).is_floor() || objects.lookup_at(pos).is_some() {
            return false;
        }

        if !check_neighborhood {
            return self.alive.lookup_at(pos).is_none();
        }

        for x in -1..=1 {
            for y in -1..=1 {
                if self.alive.lookup_at(pos + IVec2::new(x, y)).is_some() {
                    return false;
                }
            }
        }

        true
    }

    pub fn kill(
        &mut self,
        clock: &Clock,
        events: &mut Events,
        lives: &mut Lives,
        policy: &Policy,
        mut killed: Box<AliveBot>,
        reason: String,
        killer: Option<BotId>,
    ) {
        trace!(id=?killed, ?reason, ?killer, "killing bot");

        events.add(Event::BotDied { id: killed.id });
        lives.on_bot_died(clock, killed.id, killed.age());

        if let Some(id) = killer {
            events.add(Event::BotScored { id });
            lives.on_bot_scored(id);
        }

        killed.events.add(clock, &*reason);

        let decision = if !killed.oneshot
            && policy.auto_respawn
            && self.queued.len() < policy.max_queued_bots as usize
        {
            Decision::Requeue
        } else {
            Decision::Discard
        };

        match decision {
            Decision::Requeue => {
                killed.events.add(clock, "awaiting reincarnation");

                self.queued.push_back(Box::new(QueuedBot {
                    dir: None,
                    events: killed.body.events,
                    fw: killed.fw,
                    id: killed.body.id,
                    oneshot: false,
                    pos: None,
                    requeued: true,
                    serial: killed.body.serial,
                }));
            }

            Decision::Discard => {
                let bot = DeadBot {
                    events: killed.events.snapshot(),
                    id: killed.id,
                    serial: killed.serial.snapshot(),
                };

                if let Some(id) = self.dead.add(bot) {
                    events.add(Event::BotDiscarded { id });
                    lives.on_bot_discarded(id);
                }
            }
        }
    }

    pub fn contains(&self, id: BotId) -> bool {
        self.alive.contains(id)
            || self.dead.contains(id)
            || self.queued.contains(id)
    }

    pub fn remove(&mut self, id: BotId) {
        self.alive.remove(id);
        self.dead.remove(id);
        self.queued.remove(id);
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Spawn {
    pub pos: Option<IVec2>,
    pub dir: Option<Dir>,
}

#[derive(Clone, Copy, Debug)]
enum Decision {
    Requeue,
    Discard,
}
