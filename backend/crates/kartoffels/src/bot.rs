mod arm;
mod battery;
mod motor;
mod radar;
mod serial;
mod tick;
mod timer;

pub use self::arm::*;
pub use self::battery::*;
pub use self::motor::*;
pub use self::radar::*;
pub use self::serial::*;
pub use self::tick::*;
pub use self::timer::*;
use crate::{AliveBotsLocator, BotId, Map};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use glam::IVec2;
use kartoffels_vm as vm;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::mem;
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default))]
pub struct AliveBot {
    pub vm: Option<vm::Runtime>,
    pub timer: BotTimer,
    pub battery: BotBattery,
    pub serial: BotSerial,
    pub motor: BotMotor,
    pub arm: BotArm,
    pub radar: BotRadar,
}

impl AliveBot {
    const MEM_TIMER: u32 = 0;
    const MEM_BATTERY: u32 = 1024;
    const MEM_SERIAL: u32 = 2 * 1024;
    const MEM_MOTOR: u32 = 3 * 1024;
    const MEM_ARM: u32 = 4 * 1024;
    const MEM_RADAR: u32 = 5 * 1024;

    pub fn new(rng: &mut impl RngCore, vm: vm::Runtime) -> Self {
        Self {
            vm: Some(vm),
            timer: BotTimer::new(rng),
            battery: BotBattery::default(),
            serial: BotSerial::default(),
            motor: BotMotor::new(rng),
            arm: BotArm::default(),
            radar: BotRadar::default(),
        }
    }

    pub fn tick(
        &mut self,
        map: &Map,
        bots: &AliveBotsLocator,
        pos: IVec2,
    ) -> Result<AliveBotTick> {
        self.timer.tick();
        self.serial.tick();
        self.arm.tick();
        self.motor.tick();
        self.radar.tick(map, bots, pos, self.motor.dir);

        // ---

        let mut vm =
            mem::take(&mut self.vm).context("tried to tick() a crashed bot")?;

        vm.tick(self).context("firmware crashed")?;

        self.vm = Some(vm);

        // ---

        let stab_dir = if mem::take(&mut self.arm.is_stabbing) {
            Some(self.motor.dir)
        } else {
            None
        };

        let move_dir = if mem::take(&mut self.motor.vel) == 1 {
            Some(self.motor.dir)
        } else {
            None
        };

        Ok(AliveBotTick { stab_dir, move_dir })
    }

    pub fn reset(mut self, rng: &mut impl RngCore) -> Option<Self> {
        Some(Self::new(rng, self.vm.take()?.reset()))
    }
}

impl vm::Mmio for AliveBot {
    fn load(&self, addr: u32) -> Result<u32, ()> {
        self.timer
            .mmio_load(addr)
            .or_else(|_| self.battery.mmio_load(addr))
            .or_else(|_| self.serial.mmio_load(addr))
            .or_else(|_| self.motor.mmio_load(addr))
            .or_else(|_| self.arm.mmio_load(addr))
            .or_else(|_| self.radar.mmio_load(addr))
    }

    fn store(&mut self, addr: u32, val: u32) -> Result<(), ()> {
        self.timer
            .mmio_store(addr, val)
            .or_else(|_| self.battery.mmio_store(addr, val))
            .or_else(|_| self.serial.mmio_store(addr, val))
            .or_else(|_| self.motor.mmio_store(addr, val))
            .or_else(|_| self.arm.mmio_store(addr, val))
            .or_else(|_| self.radar.mmio_store(addr, val))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeadBot {
    pub reason: Arc<String>,
    pub killed_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueuedBot {
    pub id: BotId,
    pub requeued: bool,

    #[serde(flatten)]
    pub bot: AliveBot,
}
