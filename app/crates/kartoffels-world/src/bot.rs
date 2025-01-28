mod action;
mod arm;
mod battery;
mod compass;
mod events;
mod id;
mod inventory;
mod mmio;
mod motor;
mod radar;
mod serial;
mod timer;

pub use self::action::*;
pub use self::arm::*;
pub use self::battery::*;
pub use self::compass::*;
pub use self::events::*;
pub use self::id::*;
pub use self::inventory::*;
pub use self::mmio::*;
pub use self::motor::*;
pub use self::radar::*;
pub use self::serial::*;
pub use self::timer::*;
use crate::{AliveBots, Clock, Dir, Map, Objects, Ticks, WorldRng};
use glam::IVec2;
use kartoffels_cpu::{Cpu, Firmware};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default))]
pub struct AliveBot {
    pub arm: BotArm,
    pub battery: BotBattery,
    pub compass: BotCompass,
    pub cpu: Cpu,
    pub dir: Dir,
    pub events: BotEvents,
    pub fw: Firmware,
    pub id: BotId,
    pub inventory: BotInventory,
    pub motor: BotMotor,
    pub oneshot: bool,
    pub pos: IVec2,
    pub radar: BotRadar,
    pub serial: BotSerial,
    pub timer: BotTimer,
}

impl AliveBot {
    const MEM_TIMER: u32 = 0;
    const MEM_BATTERY: u32 = 1024;
    const MEM_SERIAL: u32 = 2 * 1024;
    const MEM_MOTOR: u32 = 3 * 1024;
    const MEM_ARM: u32 = 4 * 1024;
    const MEM_RADAR: u32 = 5 * 1024;
    const MEM_COMPASS: u32 = 6 * 1024;

    pub fn new(
        rng: &mut impl RngCore,
        clock: &Clock,
        pos: IVec2,
        dir: Dir,
        mut bot: QueuedBot,
    ) -> Self {
        bot.events
            .add(clock, if bot.requeued { "reincarnated" } else { "born" });

        Self {
            arm: Default::default(),
            battery: Default::default(),
            compass: Default::default(),
            cpu: Cpu::new(&bot.fw),
            dir,
            events: bot.events,
            fw: bot.fw,
            id: bot.id,
            inventory: Default::default(),
            motor: Default::default(),
            oneshot: bot.oneshot,
            pos,
            radar: Default::default(),
            serial: Default::default(),
            timer: BotTimer::new(rng),
        }
    }

    pub fn log(&mut self, clock: &Clock, msg: impl Into<String>) {
        self.events.add(clock, msg);
    }

    pub fn age(&self) -> Ticks {
        Ticks::new(self.timer.ticks())
    }

    pub fn tick(
        &mut self,
        bots: &AliveBots,
        map: &Map,
        objects: &Objects,
        rng: &mut WorldRng,
    ) -> Result<Option<BotAction>, Box<str>> {
        let mut action = None;

        self.timer.tick();
        self.serial.tick();
        self.arm.tick();
        self.motor.tick();
        self.radar.tick();
        self.compass.tick(self.dir);

        self.cpu.tick(BotMmio {
            arm: &mut self.arm,
            battery: &mut self.battery,
            compass: &mut self.compass,
            motor: &mut self.motor,
            radar: &mut self.radar,
            serial: &mut self.serial,
            timer: &mut self.timer,

            ctxt: BotMmioContext {
                action: &mut action,
                bots,
                dir: &mut self.dir,
                map,
                objects,
                pos: self.pos,
                rng: &mut rng.0,
            },
        })?;

        Ok(action)
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
    pub dir: Option<Dir>,
    pub events: BotEvents,
    pub fw: Firmware,
    pub id: BotId,
    pub oneshot: bool,
    pub pos: Option<IVec2>,
    pub requeued: bool, // TODO rename to `reincarnated`
    pub serial: BotSerial,
}
