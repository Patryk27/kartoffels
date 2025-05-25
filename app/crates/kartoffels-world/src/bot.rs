mod arm;
mod clock;
mod compass;
mod events;
mod id;
mod inventory;
mod irq;
mod motor;
mod radar;
mod serial;

pub use self::arm::*;
pub use self::clock::*;
pub use self::compass::*;
pub use self::events::*;
pub use self::id::*;
pub use self::inventory::*;
pub use self::irq::*;
pub use self::motor::*;
pub use self::radar::*;
pub use self::serial::*;
use crate::{AbsDir, Clock, Ticks, World};
use glam::{IVec2, ivec2};
use kartoffels_cpu::{Atomic, Cpu, Firmware, Mmio, TickError};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::ops;
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default))]
pub struct AliveBot {
    #[serde(flatten)]
    pub body: AliveBotBody,
    pub cpu: Cpu,
    pub fw: Firmware,
}

impl AliveBot {
    pub fn new(
        rng: &mut impl RngCore,
        clock: &Clock,
        pos: IVec2,
        dir: AbsDir,
        mut bot: QueuedBot,
    ) -> Self {
        bot.events
            .add(clock, if bot.requeued { "reincarnated" } else { "born" });

        Self {
            body: AliveBotBody {
                arm: Default::default(),
                clock: BotClock::new(rng),
                compass: Default::default(),
                dir,
                events: bot.events,
                id: bot.id,
                inventory: Default::default(),
                irq: Default::default(),
                motor: Default::default(),
                oneshot: bot.oneshot,
                pos,
                radar: Default::default(),
                serial: Default::default(),
            },

            cpu: Cpu::new(&bot.fw),
            fw: bot.fw,
        }
    }

    pub fn age(&self) -> Ticks {
        Ticks::new(self.clock.ticks())
    }

    pub fn tick(&mut self, world: &mut World) -> Result<(), TickError> {
        BotClock::tick(self);
        BotArm::tick(self);
        BotMotor::tick(self);
        BotRadar::tick(self);
        BotCompass::tick(self);
        BotIrq::tick(self);

        self.cpu.tick(BotMmio {
            bot: &mut self.body,
            world,
        })?;

        Ok(())
    }
}

impl ops::Deref for AliveBot {
    type Target = AliveBotBody;

    fn deref(&self) -> &Self::Target {
        &self.body
    }
}

impl ops::DerefMut for AliveBot {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.body
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default))]
pub struct AliveBotBody {
    pub arm: BotArm,
    pub clock: BotClock,
    pub compass: BotCompass,
    pub dir: AbsDir,
    pub events: BotEvents,
    pub id: BotId,
    pub inventory: BotInventory,
    pub irq: BotIrq,
    pub motor: BotMotor,
    pub oneshot: bool,
    pub pos: IVec2,
    pub radar: BotRadar,
    pub serial: BotSerial,
}

impl AliveBotBody {
    pub(crate) const FELL_INTO_VOID: IVec2 = ivec2(i32::MAX, i32::MAX);

    fn load(&mut self, addr: u32) -> Result<u32, ()> {
        BotClock::load(self, addr)
            .or_else(|_| BotIrq::load(self, addr))
            .or_else(|_| BotMotor::load(self, addr))
            .or_else(|_| BotArm::load(self, addr))
            .or_else(|_| BotRadar::load(self, addr))
            .or_else(|_| BotCompass::load(self, addr))
    }

    fn store(
        &mut self,
        world: &mut World,
        addr: u32,
        val: u32,
    ) -> Result<(), ()> {
        BotClock::store(self, addr, val)
            .or_else(|_| BotIrq::store(self, addr, val))
            .or_else(|_| BotSerial::store(self, addr, val))
            .or_else(|_| BotMotor::store(self, world, addr, val))
            .or_else(|_| BotArm::store(self, world, addr, val))
            .or_else(|_| BotRadar::store(self, world, addr, val))
    }

    fn atomic(&mut self, addr: u32, rhs: u32, op: Atomic) -> Result<u32, ()> {
        BotIrq::atomic(self, addr, rhs, op)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeadBot {
    pub events: Arc<VecDeque<Arc<BotEvent>>>,
    pub id: BotId,
    pub serial: Arc<VecDeque<u32>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueuedBot {
    pub dir: Option<AbsDir>,
    pub events: BotEvents,
    pub fw: Firmware,
    pub id: BotId,
    pub oneshot: bool,
    pub pos: Option<IVec2>,
    pub requeued: bool, // TODO rename to `reincarnated`
    pub serial: BotSerial,
}

struct BotMmio<'a> {
    bot: &'a mut AliveBotBody,
    world: &'a mut World,
}

impl Mmio for BotMmio<'_> {
    fn load(self, addr: u32) -> Result<u32, ()> {
        self.bot.load(addr)
    }

    fn store(self, addr: u32, val: u32) -> Result<(), ()> {
        self.bot.store(self.world, addr, val)
    }

    fn atomic(self, addr: u32, rhs: u32, op: Atomic) -> Result<u32, ()> {
        self.bot.atomic(addr, rhs, op)
    }
}
