use super::{Mmio, Runtime};
use anyhow::{anyhow, Error, Result};

impl Runtime {
    pub(super) fn mem_load<const BYTES: usize>(
        &self,
        mmio: &mut impl Mmio,
        addr: u64,
    ) -> Result<i64> {
        let addr = translate(addr, BYTES)?;

        if addr >= Self::MMIO_BASE {
            self.mem_load_mmio::<BYTES>(mmio, addr)
        } else if addr >= Self::RAM_BASE {
            self.mem_load_ram::<BYTES>(addr)
        } else if addr == 0 {
            Err(fault("null-pointer load", addr, BYTES))
        } else {
            Err(fault("out-of-bounds load", addr, BYTES))
        }
    }

    fn mem_load_mmio<const BYTES: usize>(
        &self,
        mmio: &mut impl Mmio,
        addr: u32,
    ) -> Result<i64> {
        if BYTES == 4 {
            let rel_addr = addr - Self::MMIO_BASE;

            let val = mmio
                .load(rel_addr)
                .map_err(|_| fault("out-of-bounds mmio load", addr, BYTES))?;

            Ok(val as i32 as i64)
        } else {
            Err(fault("unaligned mmio load", addr, BYTES))
        }
    }

    fn mem_load_ram<const BYTES: usize>(&self, addr: u32) -> Result<i64> {
        let rel_addr = (addr - Self::RAM_BASE) as usize;

        if rel_addr + BYTES > self.ram.len() {
            return Err(fault("out-of-bounds ram load", addr, BYTES));
        }

        let mut val = 0;

        for offset in 0..BYTES {
            val |= (self.ram[rel_addr + offset] as u64) << (offset * 8);
        }

        Ok(val as i64)
    }

    pub(super) fn mem_store<const BYTES: usize>(
        &mut self,
        mmio: &mut impl Mmio,
        addr: u64,
        val: i64,
    ) -> Result<()> {
        let addr = translate(addr, BYTES)?;
        let val = val as u64;

        if addr >= Self::MMIO_BASE {
            self.mem_store_mmio::<BYTES>(mmio, addr, val)
        } else if addr >= Self::RAM_BASE {
            self.mem_store_ram::<BYTES>(addr, val)
        } else if addr == 0 {
            Err(fault("null-pointer store", addr, BYTES))
        } else {
            Err(fault("out-of-bounds store", addr, BYTES))
        }
    }

    fn mem_store_mmio<const BYTES: usize>(
        &mut self,
        mmio: &mut impl Mmio,
        addr: u32,
        val: u64,
    ) -> Result<()> {
        if BYTES == 4 {
            let rel_addr = addr - Self::MMIO_BASE;
            let val = val as u32;

            mmio.store(rel_addr, val)
                .map_err(|_| fault("out-of-bounds mmio store", addr, BYTES))
        } else {
            Err(fault("unaligned mmio store", addr, BYTES))
        }
    }

    fn mem_store_ram<const BYTES: usize>(
        &mut self,
        addr: u32,
        val: u64,
    ) -> Result<()> {
        let rel_addr = (addr - Self::RAM_BASE) as usize;

        if rel_addr + BYTES > self.ram.len() {
            return Err(fault("out-of-bounds ram store", addr, BYTES));
        }

        for offset in 0..BYTES {
            self.ram[rel_addr + offset] = ((val >> (offset * 8)) & 0xff) as u8;
        }

        Ok(())
    }
}

fn translate(addr: u64, bytes: usize) -> Result<u32> {
    u32::try_from(addr).map_err(|_| {
        anyhow!(
            "cannot translate 0x{:16x}+{} to a 32-bit address",
            addr,
            bytes
        )
    })
}

fn fault(msg: &str, addr: u32, bytes: usize) -> Error {
    anyhow!("{} at address 0x{:08x}+{}", msg, addr, bytes)
}
