#![allow(clippy::result_unit_err)]

mod fw;
mod mem;
mod mmio;
mod tick;

pub use self::fw::*;
pub use self::mmio::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Cpu {
    pc: u64,
    #[serde(with = "serde_bytes")]
    ram: Box<[u8]>,
    regs: Box<[i64; 32]>,
}

impl Cpu {
    const RAM_BASE: u32 = 0x00100000;
    const RAM_SIZE: u32 = 128 * 1024;
    const MMIO_BASE: u32 = 0x08000000;

    pub fn new(fw: &Firmware) -> Self {
        let pc = fw.entry_pc;

        let ram = {
            let mut ram = vec![0; Self::RAM_SIZE as usize].into_boxed_slice();

            for seg in &fw.segments {
                // Unwrap-safety: `Firmware::new()` already checks the bounds
                ram[seg.addr..seg.addr + seg.data.len()]
                    .copy_from_slice(&seg.data);
            }

            ram
        };

        let regs = Box::new([0; 32]);

        Self { pc, ram, regs }
    }

    pub fn tick(&mut self, mmio: impl Mmio) -> Result<(), Box<str>> {
        self.do_tick(mmio)
    }

    pub fn try_tick(&mut self, mmio: impl Mmio) -> Result<bool, Box<str>> {
        match self.tick(mmio) {
            Ok(()) => Ok(true),
            Err(err) if err.contains("got `ebreak`") => Ok(false),
            Err(err) => Err(err),
        }
    }

    pub fn pc(&self) -> u64 {
        self.pc
    }

    pub fn ram(&self) -> &[u8] {
        &self.ram
    }

    pub fn regs(&self) -> &[i64; 32] {
        &self.regs
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cpu").field("pc", &self.pc).finish()
    }
}
