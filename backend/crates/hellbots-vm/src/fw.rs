use crate::Runtime;
use anyhow::{anyhow, Context, Error, Result};
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
                    return Err(err_segment_addr(seg_idx));
                }

                let addr = (addr - (Runtime::RAM_BASE as u64)) as u32;

                if addr + (data.len() as u32) >= Runtime::RAM_SIZE {
                    return Err(err_segment_addr(seg_idx));
                }

                segments.push(Segment {
                    addr: addr as usize,
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

fn err_segment_addr(seg_idx: usize) -> Error {
    anyhow!("segment #{} spans outside the available memory", seg_idx)
}
