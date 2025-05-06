use crate::{Cpu, FwError, FwResult};
use object::{Endian, Endianness, File, Object, ObjectSegment};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Firmware {
    entry_pc: u32,
    segments: Vec<Segment>,
}

impl Firmware {
    pub fn from_elf(src: &[u8]) -> FwResult<Self> {
        let src = match File::parse(src)? {
            File::Elf32(src) => src,
            _ => return Err(FwError::MismatchedArchitecture),
        };

        if !src.endianness().is_little_endian() {
            return Err(FwError::MismatchedEndianess);
        }

        let entry_pc = src.elf_header().e_entry.get(Endianness::Little);
        let mut segments = Vec::new();

        for (idx, seg) in src.segments().enumerate() {
            let data = seg.data()?;
            let addr = seg.elf_program_header().p_vaddr.get(Endianness::Little);

            let addr = addr.checked_sub(Cpu::RAM_BASE).ok_or(
                FwError::SegmentUnderflow {
                    idx,
                    addr,
                    limit: Cpu::RAM_BASE,
                },
            )?;

            let end_addr = addr + (data.len() as u32);

            if end_addr >= Cpu::RAM_SIZE {
                return Err(FwError::SegmentOverflow {
                    idx,
                    addr: Cpu::RAM_BASE + end_addr,
                    limit: Cpu::RAM_BASE + Cpu::RAM_SIZE - 1,
                });
            }

            segments.push(Segment {
                addr: addr as usize,
                data: data.into(),
            });
        }

        if segments.is_empty() {
            Err(FwError::NoSegments)
        } else {
            Ok(Self { entry_pc, segments })
        }
    }

    pub(crate) fn boot(&self) -> (Box<[u8]>, u32) {
        let mut ram = vec![0; Cpu::RAM_SIZE as usize].into_boxed_slice();

        for seg in &self.segments {
            // Unwrap-safety: We check bounds over `Firmware::new()`
            ram[seg.addr..seg.addr + seg.data.len()].copy_from_slice(&seg.data);
        }

        (ram, self.entry_pc)
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
    addr: usize,
    #[serde(with = "serde_bytes")]
    data: Box<[u8]>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use object::build::elf::Builder;
    use object::elf::PT_LOAD;

    trait BuilderExt
    where
        Self: Sized,
    {
        fn build(self) -> Vec<u8>;
    }

    impl BuilderExt for Builder<'_> {
        fn build(self) -> Vec<u8> {
            let mut src = Vec::new();

            self.write(&mut src).unwrap();
            src
        }
    }

    #[test]
    fn err_mismatched_architecture() {
        let src = Builder::new(Endianness::Little, true).build();
        let actual = Firmware::from_elf(&src).unwrap_err();

        assert_eq!(FwError::MismatchedArchitecture, actual);
    }

    #[test]
    fn err_mismatched_endianess() {
        let src = Builder::new(Endianness::Big, false).build();
        let actual = Firmware::from_elf(&src).unwrap_err();

        assert_eq!(FwError::MismatchedEndianess, actual);
    }

    #[test]
    fn err_no_segments() {
        let src = Builder::new(Endianness::Little, false).build();
        let actual = Firmware::from_elf(&src).unwrap_err();

        assert_eq!(FwError::NoSegments, actual);
    }

    #[test]
    fn err_segment_underflow() {
        let mut src = Builder::new(Endianness::Little, false);

        src.header.e_phoff = 52;

        let seg = src.segments.add();

        seg.p_type = PT_LOAD;
        seg.p_vaddr = 1234;

        let actual = Firmware::from_elf(&src.build()).unwrap_err();

        assert_eq!(
            FwError::SegmentUnderflow {
                idx: 0,
                addr: 1234,
                limit: Cpu::RAM_BASE,
            },
            actual
        );
    }

    #[test]
    fn err_segment_overflow() {
        let mut src = Builder::new(Endianness::Little, false);

        src.header.e_phoff = 52;

        let seg = src.segments.add();

        seg.p_type = PT_LOAD;
        seg.p_vaddr = (Cpu::RAM_BASE + Cpu::RAM_SIZE) as u64;
        seg.p_memsz = 1;

        let actual = Firmware::from_elf(&src.build()).unwrap_err();

        assert_eq!(
            FwError::SegmentOverflow {
                idx: 0,
                addr: Cpu::RAM_BASE + Cpu::RAM_SIZE,
                limit: Cpu::RAM_BASE + Cpu::RAM_SIZE - 1,
            },
            actual
        );
    }

    #[test]
    fn ok() {
        let mut src = Builder::new(Endianness::Little, false);

        src.header.e_phoff = 52;
        src.header.e_entry = 1234;

        let seg = src.segments.add();

        seg.p_type = PT_LOAD;
        seg.p_vaddr = Cpu::RAM_BASE as u64;
        seg.p_memsz = Cpu::RAM_SIZE as u64;

        let (_, ip) = Firmware::from_elf(&src.build()).unwrap().boot();

        assert_eq!(1234, ip);
    }
}
