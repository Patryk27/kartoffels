use super::{Cpu, Mmio};

impl Cpu {
    pub(super) fn mem_load<M, const SIZE: usize>(
        &self,
        mmio: Option<M>,
        addr: u32,
    ) -> Result<i32, Box<str>>
    where
        M: Mmio,
    {
        if addr >= Self::MMIO_BASE {
            let mmio = mmio.ok_or_else(|| {
                Self::mem_fault("atomic mmio load", addr, SIZE)
            })?;

            return self.mem_load_mmio::<SIZE>(mmio, addr);
        }

        if addr >= Self::RAM_BASE {
            return self.mem_load_ram::<SIZE>(addr);
        }

        if addr == 0 {
            return Err(Self::mem_fault("null-pointer load", addr, SIZE));
        }

        Err(Self::mem_fault("out-of-bounds load", addr, SIZE))
    }

    fn mem_load_mmio<const SIZE: usize>(
        &self,
        mmio: impl Mmio,
        addr: u32,
    ) -> Result<i32, Box<str>> {
        if SIZE != 4 {
            return Err(Self::mem_fault("missized mmio load", addr, SIZE));
        }

        if addr % 4 != 0 {
            return Err(Self::mem_fault("unaligned mmio load", addr, SIZE));
        }

        let rel_addr = addr - Self::MMIO_BASE;

        let val = mmio.load(rel_addr).map_err(|_| {
            Self::mem_fault("out-of-bounds mmio load", addr, SIZE)
        })?;

        Ok(val as i32)
    }

    fn mem_load_ram<const SIZE: usize>(
        &self,
        addr: u32,
    ) -> Result<i32, Box<str>> {
        let rel_addr = (addr - Self::RAM_BASE) as usize;

        if rel_addr + SIZE > self.ram.len() {
            return Err(Self::mem_fault("out-of-bounds ram load", addr, SIZE));
        }

        let mut val = 0;

        for offset in 0..SIZE {
            val |= (self.ram[rel_addr + offset] as u32) << (offset * 8);
        }

        Ok(val as i32)
    }

    pub(super) fn mem_store<M, const SIZE: usize>(
        &mut self,
        mmio: Option<M>,
        addr: u32,
        val: i32,
    ) -> Result<(), Box<str>>
    where
        M: Mmio,
    {
        if addr >= Self::MMIO_BASE {
            let mmio = mmio.ok_or_else(|| {
                Self::mem_fault("atomic mmio store", addr, SIZE)
            })?;

            return self.mem_store_mmio::<SIZE>(mmio, addr, val);
        }

        if addr >= Self::RAM_BASE {
            return self.mem_store_ram::<SIZE>(addr, val);
        }

        if addr == 0 {
            return Err(Self::mem_fault("null-pointer store", addr, SIZE));
        }

        Err(Self::mem_fault("out-of-bounds store", addr, SIZE))
    }

    fn mem_store_mmio<const SIZE: usize>(
        &mut self,
        mmio: impl Mmio,
        addr: u32,
        val: i32,
    ) -> Result<(), Box<str>> {
        if SIZE != 4 {
            return Err(Self::mem_fault("missized mmio store", addr, SIZE));
        }

        if addr % 4 != 0 {
            return Err(Self::mem_fault("unaligned mmio store", addr, SIZE));
        }

        let rel_addr = addr - Self::MMIO_BASE;

        mmio.store(rel_addr, val as u32).map_err(|_| {
            Self::mem_fault("out-of-bounds mmio store", addr, SIZE)
        })
    }

    fn mem_store_ram<const SIZE: usize>(
        &mut self,
        addr: u32,
        val: i32,
    ) -> Result<(), Box<str>> {
        let rel_addr = (addr - Self::RAM_BASE) as usize;

        if rel_addr + SIZE > self.ram.len() {
            return Err(Self::mem_fault("out-of-bounds ram store", addr, SIZE));
        }

        let val = val as u32;

        for offset in 0..SIZE {
            self.ram[rel_addr + offset] = (val >> (offset * 8)) as u8;
        }

        Ok(())
    }

    fn mem_fault(msg: &str, addr: u32, size: usize) -> Box<str> {
        format!("{msg} on 0x{addr:08x}+{size}").into()
    }
}
