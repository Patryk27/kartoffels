mod arm;
mod battery;
mod events;
mod id;
mod mmio;
mod motor;
mod radar;
mod serial;
mod tick;
mod timer;

pub use self::arm::*;
pub use self::battery::*;
pub use self::events::*;
pub use self::id::*;
pub use self::mmio::*;
pub use self::motor::*;
pub use self::radar::*;
pub use self::serial::*;
pub use self::tick::*;
pub use self::timer::*;
use crate::{AliveBotsLocator, Dir, Map};
use glam::IVec2;
use kartoffels_cpu::Cpu;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::mem;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default))]
pub struct AliveBot {
    pub arm: BotArm,
    pub battery: BotBattery,
    pub cpu: Cpu,
    pub dir: Dir,
    pub events: BotEvents,
    pub motor: BotMotor,
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
            cpu: bot.cpu,
            dir,
            events: bot.events,
            motor: Default::default(),
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
    ) -> Result<AliveBotTick, String> {
        self.timer.tick();
        self.serial.tick();
        self.arm.tick();
        self.motor.tick();
        self.radar.tick(map, bots, self.pos, self.dir);

        self.cpu.tick(&mut BotMmio {
            timer: &mut self.timer,
            battery: &mut self.battery,
            serial: &mut self.serial,
            motor: &mut self.motor,
            arm: &mut self.arm,
            radar: &mut self.radar,
            ctxt: BotMmioContext {
                dir: &mut self.dir,
                rng: &mut *rng,
            },
        })?;

        // ---

        let stab_dir = if mem::take(&mut self.arm.is_stabbing) {
            Some(self.dir)
        } else {
            None
        };

        let move_dir = if mem::take(&mut self.motor.vel) == 1 {
            Some(self.dir)
        } else {
            None
        };

        Ok(AliveBotTick {
            pos: self.pos,
            stab_dir,
            move_dir,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeadBot {
    pub events: BotEvents,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueuedBot {
    pub id: BotId,
    pub pos: Option<IVec2>,
    pub requeued: bool,

    pub cpu: Cpu,
    pub events: BotEvents,
    pub serial: BotSerial,
}
