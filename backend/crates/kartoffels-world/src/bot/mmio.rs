use super::{BotArm, BotBattery, BotMotor, BotRadar, BotSerial, BotTimer};
use crate::Dir;
use kartoffels_cpu::Mmio;
use rand::{Rng, RngCore};

pub struct BotMmio<'a> {
    pub arm: &'a mut BotArm,
    pub battery: &'a mut BotBattery,
    pub motor: &'a mut BotMotor,
    pub radar: &'a mut BotRadar,
    pub serial: &'a mut BotSerial,
    pub timer: &'a mut BotTimer,
    pub ctxt: BotMmioContext<'a>,
}

impl<'a> Mmio for BotMmio<'a> {
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
            .or_else(|_| self.motor.mmio_store(&mut self.ctxt, addr, val))
            .or_else(|_| self.arm.mmio_store(&mut self.ctxt, addr, val))
            .or_else(|_| self.radar.mmio_store(&mut self.ctxt, addr, val))
    }
}

pub struct BotMmioContext<'a> {
    pub dir: &'a mut Dir,
    pub rng: &'a mut dyn RngCore,
}

impl BotMmioContext<'_> {
    pub fn cooldown(&mut self, base: u32, off_percentage: u32) -> u32 {
        let off = base * off_percentage / 100;
        let min = base - off;
        let max = base + off;

        self.rng.gen_range(min..=max)
    }
}
