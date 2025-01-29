use crate::Cpu;
use anyhow::{anyhow, Context, Result};
use elf::abi::PT_LOAD;
use elf::endian::LittleEndian;
use elf::file::Class;
use elf::ElfBytes;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Firmware {
    pub(crate) segments: Vec<Segment>,
    pub(crate) entry_pc: u32,
}

impl Firmware {
    pub fn from_elf(src: &[u8]) -> Result<Self> {
        let mut segments = Vec::new();

        let elf = ElfBytes::<LittleEndian>::minimal_parse(src)?;
        let entry_pc = elf.ehdr.e_entry as u32;

        if elf.ehdr.class == Class::ELF64 {
            return Err(anyhow!(
                "expected a 32-bit binary, but got a 64-bit one\n\n\
                 this is most likely the outcome of a backwards-incompatible \
                 change introduced in kartoffels v0.7 - if you're following \
                 the kartoffel repository, simply clone it again and copy your \
                 code there\n\n\
                 sorry for the trouble and godspeed!",
            ));
        }

        for (seg_idx, seg) in elf
            .segments()
            .context("found no segments")?
            .into_iter()
            .enumerate()
        {
            if seg.p_type == PT_LOAD {
                let addr = seg.p_vaddr;
                let data = elf.segment_data(&seg)?;

                if addr < (Cpu::RAM_BASE as u64) {
                    return Err(anyhow!(
                        "segment #{} spans outside the available memory (it \
                         starts at 0x{:0x}, which is before 0x{:0x})",
                        seg_idx,
                        addr,
                        Cpu::RAM_BASE,
                    ));
                }

                let beg_addr = (addr - (Cpu::RAM_BASE as u64)) as u32;
                let end_addr = beg_addr + (data.len() as u32);

                if end_addr >= Cpu::RAM_SIZE {
                    return Err(anyhow!(
                        "segment #{} spans outside the available memory (it \
                         ends at 0x{:0x}, which is after 0x{:0x})",
                        seg_idx,
                        Cpu::RAM_BASE + end_addr,
                        Cpu::RAM_SIZE,
                    ));
                }

                segments.push(Segment {
                    addr: beg_addr as usize,
                    data: data.into(),
                });
            }
        }

        Ok(Self { segments, entry_pc })
    }
}

impl fmt::Debug for Firmware {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Firmware")
            .field("entry_pc", &self.entry_pc)
            .finish()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Segment {
    pub(super) addr: usize,
    #[serde(with = "serde_bytes")]
    pub(super) data: Box<[u8]>,
}
