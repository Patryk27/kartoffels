mod action;
mod arm;
mod battery;
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
pub use self::events::*;
pub use self::id::*;
pub use self::inventory::*;
pub use self::mmio::*;
pub use self::motor::*;
pub use self::radar::*;
pub use self::serial::*;
pub use self::timer::*;
use crate::{AliveBotsLocator, Dir, Map};
use glam::IVec2;
use kartoffels_cpu::{Cpu, Firmware};
use rand::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default))]
pub struct AliveBot {
    pub arm: BotArm,
    pub battery: BotBattery,
    pub cpu: Cpu,
    pub dir: Dir,
    pub events: BotEvents,
    pub fw: Firmware,
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

    pub fn spawn(
        rng: &mut impl RngCore,
        pos: IVec2,
        dir: Dir,
        mut bot: QueuedBot,
    ) -> Self {
        bot.events
            .add(if bot.requeued { "respawned" } else { "spawned" });

        Self {
            arm: Default::default(),
            battery: Default::default(),
            cpu: Cpu::new(&bot.fw),
            dir,
            events: bot.events,
            fw: bot.fw,
            inventory: Default::default(),
            motor: Default::default(),
            oneshot: bot.oneshot,
            pos,
            radar: Default::default(),
            serial: Default::default(),
            timer: BotTimer::new(rng),
        }
    }

    pub fn log(&mut self, msg: impl Into<String>) {
        self.events.add(msg);
    }

    pub fn tick(
        &mut self,
        rng: &mut impl RngCore,
        map: &Map,
        bots: &AliveBotsLocator,
    ) -> Result<Option<BotAction>, Box<str>> {
        let mut action = None;

        self.timer.tick();
        self.serial.tick();
        self.arm.tick();
        self.motor.tick();
        self.radar.tick();

        self.cpu.tick(BotMmio {
            timer: &mut self.timer,
            battery: &mut self.battery,
            serial: &mut self.serial,
            motor: &mut self.motor,
            arm: &mut self.arm,
            radar: &mut self.radar,
            ctxt: BotMmioContext {
                action: &mut action,
                bots,
                dir: &mut self.dir,
                map,
                pos: self.pos,
                rng: &mut *rng,
            },
        })?;

        Ok(action)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeadBot {
    pub events: BotEvents,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueuedBot {
    pub dir: Option<Dir>,
    pub events: BotEvents,
    pub fw: Firmware,
    pub id: BotId,
    pub oneshot: bool,
    pub pos: Option<IVec2>,
    pub requeued: bool,
    pub serial: BotSerial,
}
