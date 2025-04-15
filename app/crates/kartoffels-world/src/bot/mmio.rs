use super::{
    BotAction, BotArm, BotBattery, BotCompass, BotMotor, BotRadar, BotSerial,
    BotTimer,
};
use crate::{AliveBots, Dir, Map, Objects};
use glam::IVec2;
use kartoffels_cpu::Mmio;
use rand::Rng;
use rand_chacha::ChaCha8Rng;

pub struct BotMmio<'a> {
    pub arm: &'a mut BotArm,
    pub battery: &'a mut BotBattery,
    pub compass: &'a mut BotCompass,
    pub motor: &'a mut BotMotor,
    pub radar: &'a mut BotRadar,
    pub serial: &'a mut BotSerial,
    pub timer: &'a mut BotTimer,
    pub ctxt: BotMmioContext<'a>,
}

impl Mmio for BotMmio<'_> {
    fn load(&mut self, addr: u32) -> Result<u32, ()> {
        self.timer
            .mmio_load(addr)
            .or_else(|_| self.battery.mmio_load(addr))
            .or_else(|_| self.serial.mmio_load(addr))
            .or_else(|_| self.motor.mmio_load(addr))
            .or_else(|_| self.arm.mmio_load(addr))
            .or_else(|_| self.radar.mmio_load(addr))
            .or_else(|_| self.compass.mmio_load(addr))
    }

    fn store(&mut self, addr: u32, val: u32) -> Result<(), ()> {
        self.timer
            .mmio_store(addr, val)
            .or_else(|_| self.battery.mmio_store(addr, val))
            .or_else(|_| self.serial.mmio_store(addr, val))
            .or_else(|_| self.motor.mmio_store(&mut self.ctxt, addr, val))
            .or_else(|_| self.arm.mmio_store(&mut self.ctxt, addr, val))
            .or_else(|_| self.radar.mmio_store(&mut self.ctxt, addr, val))
    }

    fn is_atomic_allowed(&self, _: u32) -> bool {
        false
    }
}

pub struct BotMmioContext<'a> {
    pub action: &'a mut Option<BotAction>,
    pub bots: &'a AliveBots,
    pub dir: &'a mut Dir,
    pub map: &'a Map,
    pub objects: &'a Objects,
    pub pos: IVec2,
    pub rng: &'a mut ChaCha8Rng,
}

impl BotMmioContext<'_> {
    pub fn cooldown(&mut self, base: u32) -> u32 {
        #[cfg(test)]
        let off = 0;

        #[cfg(not(test))]
        let off = base / 10;

        let min = base - off;
        let max = base + off;

        self.rng.gen_range(min..=max)
    }
}
