mod arm;
mod battery;
mod events;
mod id;
mod motor;
mod radar;
mod serial;
mod tick;
mod timer;

pub use self::arm::*;
pub use self::battery::*;
pub use self::events::*;
pub use self::id::*;
pub use self::motor::*;
pub use self::radar::*;
pub use self::serial::*;
pub use self::tick::*;
pub use self::timer::*;
use crate::{AliveBotsLocator, Map};
use anyhow::{Context, Result};
use glam::IVec2;
use kartoffels_vm as vm;
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};
use std::mem;

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
    pub events: BotEvents,
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
            events: BotEvents::default(),
        }
    }

    pub fn log(&mut self, msg: String) {
        self.events.add(msg);
    }

    pub fn tick(
        &mut self,
        rng: &mut impl RngCore,
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

        vm.tick(&mut BotMmio {
            bot: self,
            ctxt: BotMmioContext { rng: &mut *rng },
        })
        .context("firmware crashed")?;

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

    pub fn reset(mut self, rng: &mut impl RngCore) -> Result<Self, Self> {
        if let Some(vm) = self.vm.take() {
            let vm = vm.reset();
            let this = Self::new(rng, vm);

            Ok(Self {
                events: self.events,
                ..this
            })
        } else {
            Err(self)
        }
    }
}

pub struct BotMmio<'a> {
    pub bot: &'a mut AliveBot,
    pub ctxt: BotMmioContext<'a>,
}

impl<'a> vm::Mmio for BotMmio<'a> {
    fn load(&self, addr: u32) -> Result<u32, ()> {
        self.bot
            .timer
            .mmio_load(addr)
            .or_else(|_| self.bot.battery.mmio_load(addr))
            .or_else(|_| self.bot.serial.mmio_load(addr))
            .or_else(|_| self.bot.motor.mmio_load(addr))
            .or_else(|_| self.bot.arm.mmio_load(addr))
            .or_else(|_| self.bot.radar.mmio_load(addr))
    }

    fn store(&mut self, addr: u32, val: u32) -> Result<(), ()> {
        self.bot
            .timer
            .mmio_store(addr, val)
            .or_else(|_| self.bot.battery.mmio_store(addr, val))
            .or_else(|_| self.bot.serial.mmio_store(addr, val))
            .or_else(|_| self.bot.motor.mmio_store(&mut self.ctxt, addr, val))
            .or_else(|_| self.bot.arm.mmio_store(&mut self.ctxt, addr, val))
            .or_else(|_| self.bot.radar.mmio_store(&mut self.ctxt, addr, val))
    }
}

pub struct BotMmioContext<'a> {
    pub rng: &'a mut dyn RngCore,
}

impl BotMmioContext<'_> {
    fn cooldown(&mut self, base: u32, off_percentage: u32) -> u32 {
        let off = base * off_percentage / 100;
        let min = base - off;
        let max = base + off;

        self.rng.gen_range(min..=max)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeadBot {
    pub events: BotEvents,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueuedBot {
    pub id: BotId,
    pub requeued: bool,

    #[serde(flatten)]
    pub bot: AliveBot,
}
