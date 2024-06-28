use crate::Runtime;
use anyhow::{anyhow, Context, Result};
use elf::abi::PT_LOAD;
use elf::endian::LittleEndian;
use elf::ElfBytes;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Firmware {
    pub(super) segments: Vec<Segment>,
    pub(super) entry_pc: u64,
}

impl Firmware {
    pub fn new(src: &[u8]) -> Result<Self> {
        let mut segments = Vec::new();

        let elf = ElfBytes::<LittleEndian>::minimal_parse(src)?;
        let entry_pc = elf.ehdr.e_entry;

        for (seg_idx, seg) in elf
            .segments()
            .context("found no segments")?
            .into_iter()
            .enumerate()
        {
            if seg.p_type == PT_LOAD {
                let addr = seg.p_vaddr;
                let data = elf.segment_data(&seg)?;

                if addr < (Runtime::RAM_BASE as u64) {
                    return Err(anyhow!(
                        "segment #{} spans outside the available memory (it \
                         starts at 0x{:0x}, which is before 0x{:0x})",
                        seg_idx,
                        addr,
                        Runtime::RAM_BASE,
                    ));
                }

                let beg_addr = (addr - (Runtime::RAM_BASE as u64)) as u32;
                let end_addr = beg_addr + (data.len() as u32);

                if end_addr >= Runtime::RAM_SIZE {
                    return Err(anyhow!(
                        "segment #{} spans outside the available memory (it \
                         ends at 0x{:0x}, which is after 0x{:0x})",
                        seg_idx,
                        Runtime::RAM_BASE + end_addr,
                        Runtime::RAM_SIZE,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Segment {
    pub(super) addr: usize,
    #[serde(with = "serde_bytes")]
    pub(super) data: Box<[u8]>,
}
