use super::{Cpu, Mmio};

impl Cpu {
    pub(super) fn mem_load<const SIZE: usize>(
        &self,
        mmio: &mut dyn Mmio,
        addr: u64,
    ) -> Result<i64, Box<str>> {
        let addr = Self::mem_translate(addr, SIZE)?;

        if addr >= Self::MMIO_BASE {
            self.mem_load_mmio::<SIZE>(mmio, addr)
        } else if addr >= Self::RAM_BASE {
            self.mem_load_ram::<SIZE>(addr)
        } else if addr == 0 {
            Err(Self::mem_fault("null-pointer load", addr, SIZE))
        } else {
            Err(Self::mem_fault("out-of-bounds load", addr, SIZE))
        }
    }

    fn mem_load_mmio<const SIZE: usize>(
        &self,
        mmio: &mut dyn Mmio,
        addr: u32,
    ) -> Result<i64, Box<str>> {
        if SIZE == 4 {
            if addr % 4 != 0 {
                return Err(Self::mem_fault("unaligned mmio load", addr, SIZE));
            }

            let rel_addr = addr - Self::MMIO_BASE;

            let val = mmio.load(rel_addr).map_err(|_| {
                Self::mem_fault("out-of-bounds mmio load", addr, SIZE)
            })?;

            Ok(val as i32 as i64)
        } else {
            Err(Self::mem_fault("invalid-sized mmio load", addr, SIZE))
        }
    }

    fn mem_load_ram<const SIZE: usize>(
        &self,
        addr: u32,
    ) -> Result<i64, Box<str>> {
        let rel_addr = (addr - Self::RAM_BASE) as usize;

        if rel_addr + SIZE > self.ram.len() {
            return Err(Self::mem_fault("out-of-bounds ram load", addr, SIZE));
        }

        let mut val = 0;

        for offset in 0..SIZE {
            val |= (self.ram[rel_addr + offset] as u64) << (offset * 8);
        }

        Ok(val as i64)
    }

    pub(super) fn mem_store<const SIZE: usize>(
        &mut self,
        mmio: &mut dyn Mmio,
        addr: u64,
        val: i64,
    ) -> Result<(), Box<str>> {
        let addr = Self::mem_translate(addr, SIZE)?;
        let val = val as u64;

        if addr >= Self::MMIO_BASE {
            self.mem_store_mmio::<SIZE>(mmio, addr, val)
        } else if addr >= Self::RAM_BASE {
            self.mem_store_ram::<SIZE>(addr, val)
        } else if addr == 0 {
            Err(Self::mem_fault("null-pointer store", addr, SIZE))
        } else {
            Err(Self::mem_fault("out-of-bounds store", addr, SIZE))
        }
    }

    fn mem_store_mmio<const SIZE: usize>(
        &mut self,
        mmio: &mut dyn Mmio,
        addr: u32,
        val: u64,
    ) -> Result<(), Box<str>> {
        if SIZE == 4 {
            if addr % 4 != 0 {
                return Err(Self::mem_fault(
                    "unaligned mmio store",
                    addr,
                    SIZE,
                ));
            }

            let rel_addr = addr - Self::MMIO_BASE;
            let val = val as u32;

            mmio.store(rel_addr, val).map_err(|_| {
                Self::mem_fault("out-of-bounds mmio store", addr, SIZE)
            })
        } else {
            Err(Self::mem_fault("invalid-sized mmio store", addr, SIZE))
        }
    }

    fn mem_store_ram<const SIZE: usize>(
        &mut self,
        addr: u32,
        val: u64,
    ) -> Result<(), Box<str>> {
        let rel_addr = (addr - Self::RAM_BASE) as usize;

        if rel_addr + SIZE > self.ram.len() {
            return Err(Self::mem_fault("out-of-bounds ram store", addr, SIZE));
        }

        for offset in 0..SIZE {
            self.ram[rel_addr + offset] = ((val >> (offset * 8)) & 0xff) as u8;
        }

        Ok(())
    }

    pub(super) fn mem_translate(
        addr: u64,
        size: usize,
    ) -> Result<u32, Box<str>> {
        u32::try_from(addr).map_err(|_| {
            format!("cannot translate 0x{addr:16x}+{size} to a 32-bit address")
                .into()
        })
    }

    pub(super) fn mem_fault(msg: &str, addr: u32, size: usize) -> Box<str> {
        format!("{msg} on 0x{addr:08x}+{size}").into()
    }
}
