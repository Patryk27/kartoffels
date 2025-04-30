#![allow(clippy::result_unit_err)]

mod error;
mod firmware;

pub use self::error::*;
pub use self::firmware::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Cpu {
    pc: u32,
    #[serde(with = "serde_bytes")]
    ram: Box<[u8]>,
    regs: Box<[i32; 32]>,
}

impl Cpu {
    const RAM_BASE: u32 = 0x00100000;
    const RAM_SIZE: u32 = 128 * 1024;
    const MMIO_BASE: u32 = 0x08000000;

    pub fn new(fw: &Firmware) -> Self {
        let (pc, ram) = fw.boot();
        let regs = Box::new([0; 32]);

        Self { pc, ram, regs }
    }

    pub fn pc(&self) -> u32 {
        self.pc
    }

    pub fn ram(&self) -> &[u8] {
        &self.ram
    }

    pub fn regs(&self) -> &[i32; 32] {
        &self.regs
    }

    pub fn tick(&mut self, mmio: impl Mmio) -> TickResult<()> {
        let word = self.mem_load::<()>(None, self.pc, 4)? as u32;

        let op = word & 0x7f;
        let funct3 = (word >> 12) & 0x7;
        let funct7 = word >> 25;

        self.pc += 4;

        macro_rules! op {
            (fn $name:ident ( $($arg:ident),* ) $body:tt) => {{
                $(
                    let $arg = op!(@arg $arg);
                )*

                $body;
            }};

            (@arg rd) => {
                ((word >> 7) & 0x1f) as usize
            };

            (@arg rs1) => {
                ((word >> 15) & 0x1f) as usize
            };

            (@arg rs2) => {
                ((word >> 20) & 0x1f) as usize
            };

            (@arg i_imm) => {
                (word as i32 as i32) >> 20
            };

            (@arg u_imm) => {
                (word as i32 as i32) >> 12
            };

            (@arg s_imm) => {
                ((word & 0xfe000000) as i32 as i32 >> 20)
                    | (((word >> 7) & 0x1f) as i32)
            };

            (@arg b_imm) => {
                (((word & 0x80000000) as i32 >> 19) as u32
                    | ((word & 0x80) << 4)
                    | ((word >> 20) & 0x7e0)
                    | ((word >> 7) & 0x1e)) as i32
            };

            (@arg j_imm) => {
                (((word & 0x80000000) as i32 >> 11) as u32
                    | (word & 0xff000)
                    | ((word >> 9) & 0x800)
                    | ((word >> 20) & 0x7fe)) as i32
            };
        }

        match (op, funct3, funct7) {
            (0b0110111, _, _) => op! {
                fn lui(rd, u_imm) {
                    self.reg_store(rd, u_imm << 12);
                }
            },

            (0b0010111, _, _) => op! {
                fn auipc(rd, u_imm) {
                    self.reg_store(rd, (self.pc as i32) - 4 + (u_imm << 12));
                }
            },

            (0b0110011, 0b000, 0b0000000) => op! {
                fn add(rd, rs1, rs2) {
                    let lhs = self.regs[rs1];
                    let rhs = self.regs[rs2];

                    self.reg_store(rd, lhs.wrapping_add(rhs));
                }
            },

            (0b0010011, 0b000, _) => op! {
                fn addi(rd, rs1, i_imm) {
                    let lhs = self.regs[rs1];
                    let rhs = i_imm;

                    self.reg_store(rd, lhs.wrapping_add(rhs));
                }
            },

            (0b0110011, 0b000, 0b0100000) => op! {
                fn sub(rd, rs1, rs2) {
                    let lhs = self.regs[rs1];
                    let rhs = self.regs[rs2];

                    self.reg_store(rd, lhs.wrapping_sub(rhs));
                }
            },

            (0b0110011, 0b000, 0b0000001) => op! {
                fn mul(rd, rs1, rs2) {
                    let lhs = self.regs[rs1];
                    let rhs = self.regs[rs2];

                    self.reg_store(rd, lhs.wrapping_mul(rhs));
                }
            },

            (0b0110011, 0b001, 0b0000001) => op! {
                fn mulh(rd, rs1, rs2) {
                    let lhs = self.regs[rs1] as i64;
                    let rhs = self.regs[rs2] as i64;

                    self.reg_store(rd, (lhs.wrapping_mul(rhs) >> 32) as i32);
                }
            },

            (0b0110011, 0b010, 0b0000001) => op! {
                fn mulhsu(rd, rs1, rs2) {
                    let lhs = self.regs[rs1] as i64 as u64;
                    let rhs = self.regs[rs2] as u32 as u64;

                    self.reg_store(rd, (lhs.wrapping_mul(rhs) >> 32) as u32 as i32);
                }
            },

            (0b0110011, 0b011, 0b0000001) => op! {
                fn mulhu(rd, rs1, rs2) {
                    let lhs = self.regs[rs1] as u32 as u64;
                    let rhs = self.regs[rs2] as u32 as u64;

                    self.reg_store(rd, (lhs.wrapping_mul(rhs) >> 32) as i32);
                }
            },

            (0b0110011, 0b100, 0b0000001) => op! {
                fn div(rd, rs1, rs2) {
                    let lhs = self.regs[rs1];
                    let rhs = self.regs[rs2];

                    self.reg_store(rd, lhs.checked_div(rhs).unwrap_or(-1));
                }
            },

            (0b0110011, 0b101, 0b0000001) => op! {
                fn divu(rd, rs1, rs2) {
                    let lhs = self.regs[rs1] as u32;
                    let rhs = self.regs[rs2] as u32;

                    self.reg_store(
                        rd,
                        lhs.checked_div(rhs).unwrap_or(-1i32 as u32) as i32,
                    );
                }
            },

            (0b0110011, 0b110, 0b0000001) => op! {
                fn rem(rd, rs1, rs2) {
                    let lhs = self.regs[rs1];
                    let rhs = self.regs[rs2];

                    self.reg_store(rd, lhs.checked_rem(rhs).unwrap_or(-1));
                }
            },

            (0b0110011, 0b111, 0b0000001) => op! {
                fn remu(rd, rs1, rs2) {
                    let lhs = self.regs[rs1] as u32;
                    let rhs = self.regs[rs2] as u32;

                    self.reg_store(
                        rd,
                        lhs.checked_rem(rhs).unwrap_or(-1i32 as u32) as i32,
                    );
                }
            },

            (0b0110011, 0b111, 0b0000000) => op! {
                fn and(rd, rs1, rs2) {
                    let lhs = self.regs[rs1];
                    let rhs = self.regs[rs2];

                    self.reg_store(rd, lhs & rhs);
                }
            },

            (0b0010011, 0b111, _) => op! {
                fn andi(rd, rs1, i_imm) {
                    let lhs = self.regs[rs1];
                    let rhs = i_imm;

                    self.reg_store(rd, lhs & rhs);
                }
            },

            (0b0110011, 0b110, 0b0000000) => op! {
                fn or(rd, rs1, rs2) {
                    let lhs = self.regs[rs1];
                    let rhs = self.regs[rs2];

                    self.reg_store(rd, lhs | rhs);
                }
            },

            (0b0010011, 0b110, _) => op! {
                fn ori(rd, rs1, i_imm) {
                    let lhs = self.regs[rs1];
                    let rhs = i_imm;

                    self.reg_store(rd, lhs | rhs);
                }
            },

            (0b0110011, 0b100, 0b0000000) => op! {
                fn xor(rd, rs1, rs2) {
                    let lhs = self.regs[rs1];
                    let rhs = self.regs[rs2];

                    self.reg_store(rd, lhs ^ rhs);
                }
            },

            (0b0010011, 0b100, _) => op! {
                fn xori(rd, rs1, i_imm) {
                    let lhs = self.regs[rs1];
                    let rhs = i_imm;

                    self.reg_store(rd, lhs ^ rhs);
                }
            },

            (0b0110011, 0b001, 0b0000000) => op! {
                fn sll(rd, rs1, rs2) {
                    let lhs = self.regs[rs1] as u32;
                    let rhs = self.regs[rs2] as u32;

                    self.reg_store(rd, lhs.wrapping_shl(rhs) as i32);
                }
            },

            (0b0010011, 0b001, _) => {
                let i_imm = op!(@arg i_imm);

                match i_imm >> 6 {
                    0x00 => op! {
                        fn slli(rd, rs1, i_imm) {
                            let lhs = self.regs[rs1] as u32;
                            let rhs = i_imm as u32;

                            self.reg_store(rd, lhs.wrapping_shl(rhs) as i32);
                        }
                    },

                    _ => {
                        return Err(TickError::UnknownInstruction { word });
                    }
                }
            }

            (0b0110011, 0b101, 0b0000000) => op! {
                fn srl(rd, rs1, rs2) {
                    let lhs = self.regs[rs1] as u32;
                    let rhs = self.regs[rs2] as u32;

                    self.reg_store(rd, lhs.wrapping_shr(rhs) as i32);
                }
            },

            (0b0010011, 0b101, _) => {
                let i_imm = op!(@arg i_imm);

                match i_imm >> 6 {
                    0x00 => op! {
                        fn srli(rd, rs1, i_imm) {
                            let lhs = self.regs[rs1] as u32;
                            let rhs = i_imm as u32;

                            self.reg_store(rd, lhs.wrapping_shr(rhs) as i32);
                        }
                    },

                    0x10 => op! {
                        fn srai(rd, rs1, i_imm) {
                            let lhs = self.regs[rs1];
                            let rhs = (i_imm as u32) & 0x3f;

                            self.reg_store(rd, lhs.wrapping_shr(rhs));
                        }
                    },

                    _ => {
                        return Err(TickError::UnknownInstruction { word });
                    }
                }
            }

            (0b0110011, 0b101, 0b0100000) => op! {
                fn sra(rd, rs1, rs2) {
                    let lhs = self.regs[rs1];
                    let rhs = self.regs[rs2] as u32;

                    self.reg_store(rd, lhs.wrapping_shr(rhs));
                }
            },

            (0b0110011, 0b010, 0b0000000) => op! {
                fn slt(rd, rs1, rs2) {
                    let lhs = self.regs[rs1];
                    let rhs = self.regs[rs2];

                    self.reg_store(rd, (lhs < rhs) as i32);
                }
            },

            (0b0010011, 0b010, _) => op! {
                fn slti(rd, rs1, i_imm) {
                    let lhs = self.regs[rs1];
                    let rhs = i_imm;

                    self.reg_store(rd, (lhs < rhs) as i32);
                }
            },

            (0b0110011, 0b011, 0b0000000) => op! {
                fn sltu(rd, rs1, rs2) {
                    let lhs = self.regs[rs1] as u32;
                    let rhs = self.regs[rs2] as u32;

                    self.reg_store(rd, (lhs < rhs) as i32);
                }
            },

            (0b0010011, 0b011, _) => op! {
                fn sltiu(rd, rs1, i_imm) {
                    let lhs = self.regs[rs1] as u32;
                    let rhs = i_imm as u32;

                    self.reg_store(rd, (lhs < rhs) as i32);
                }
            },

            (0b0000011, 0b000, _) => op! {
                fn lb(rd, rs1, i_imm) {
                    let addr = (self.regs[rs1] + i_imm) as u32;
                    let value = self.mem_load(Some(mmio), addr, 1)? as i8 as i32;

                    self.reg_store(rd, value);
                }
            },

            (0b0000011, 0b100, _) => op! {
                fn lbu(rd, rs1, i_imm) {
                    let addr = (self.regs[rs1] + i_imm) as u32;
                    let value = self.mem_load(Some(mmio), addr, 1)?;

                    self.reg_store(rd, value);
                }
            },

            (0b0000011, 0b001, _) => op! {
                fn lh(rd, rs1, i_imm) {
                    let addr = (self.regs[rs1] + i_imm) as u32;
                    let value = self.mem_load(Some(mmio), addr, 2)? as i16 as i32;

                    self.reg_store(rd, value);
                }
            },

            (0b0000011, 0b101, _) => op! {
                fn lhu(rd, rs1, i_imm) {
                    let addr = (self.regs[rs1] + i_imm) as u32;
                    let value = self.mem_load(Some(mmio), addr, 2)?;

                    self.reg_store(rd, value);
                }
            },

            (0b0000011, 0b010, _) => op! {
                fn lw(rd, rs1, i_imm) {
                    let addr = (self.regs[rs1] + i_imm) as u32;
                    let value = self.mem_load(Some(mmio), addr, 4)?;

                    self.reg_store(rd, value);
                }
            },

            (0b0100011, 0b000, _) => op! {
                fn sb(rs1, rs2, s_imm) {
                    let addr = self.regs[rs1].wrapping_add(s_imm) as u32;
                    let value = self.regs[rs2] as u32;

                    self.mem_store(Some(mmio), addr, 1, value)?;
                }
            },

            (0b0100011, 0b001, _) => op! {
                fn sh(rs1, rs2, s_imm) {
                    let addr = self.regs[rs1].wrapping_add(s_imm) as u32;
                    let value = self.regs[rs2] as u32;

                    self.mem_store(Some(mmio), addr, 2, value)?;
                }
            },

            (0b0100011, 0b010, _) => op! {
                fn sw(rs1, rs2, s_imm) {
                    let addr = self.regs[rs1].wrapping_add(s_imm) as u32;
                    let value = self.regs[rs2] as u32;

                    self.mem_store(Some(mmio), addr, 4, value)?;
                }
            },

            (0b0101111, 0b010, _) => {
                // funct7's low bits encode the ordering semantics (acquire
                // and/or release) which we don't care about
                let funct5 = funct7 >> 2;

                match funct5 {
                    0b00000 => op! {
                        fn amoaddw(rd, rs1, rs2) {
                            self.tick_atomic(rd, rs1, rs2, Atomic::Add)?;
                        }
                    },

                    0b00001 => op! {
                        fn amoswapw(rd, rs1, rs2) {
                            self.tick_atomic(rd, rs1, rs2, Atomic::Swap)?;
                        }
                    },

                    0b00010 => op! {
                        fn lrw(rd, rs1) {
                            let addr = self.regs[rs1] as u32;
                            let value = self.mem_load(Some(mmio), addr, 4)?;

                            self.reg_store(rd, value);
                        }
                    },

                    0b00011 => op! {
                        fn scw(rd, rs1, rs2) {
                            let addr = self.regs[rs1] as u32;
                            let value = self.regs[rs2] as u32;

                            self.mem_store(Some(mmio), addr, 4, value)?;
                            self.regs[rd] = 0;
                        }
                    },

                    0b00100 => op! {
                        fn amoxorw(rd, rs1, rs2) {
                            self.tick_atomic(rd, rs1, rs2, Atomic::Xor)?;
                        }
                    },

                    0b01100 => op! {
                        fn amoandw(rd, rs1, rs2) {
                            self.tick_atomic(rd, rs1, rs2, Atomic::And)?;
                        }
                    },

                    0b01000 => op! {
                        fn amoorw(rd, rs1, rs2) {
                            self.tick_atomic(rd, rs1, rs2, Atomic::Or)?;
                        }
                    },

                    0b10000 => op! {
                        fn amominw(rd, rs1, rs2) {
                            self.tick_atomic(rd, rs1, rs2, Atomic::Min)?;
                        }
                    },

                    0b10100 => op! {
                        fn amomaxw(rd, rs1, rs2) {
                            self.tick_atomic(rd, rs1, rs2, Atomic::Max)?;
                        }
                    },

                    0b11000 => op! {
                        fn amominuw(rd, rs1, rs2) {
                            self.tick_atomic(rd, rs1, rs2, Atomic::MinU)?;
                        }
                    },

                    0b11100 => op! {
                        fn amomaxuw(rd, rs1, rs2) {
                            self.tick_atomic(rd, rs1, rs2, Atomic::MaxU)?;
                        }
                    },

                    _ => {
                        return Err(TickError::UnknownInstruction { word });
                    }
                }
            }

            (0b0001111, 0b000, _) => {
                // atomic fence
            }

            (0b1100011, 0b000, _) => op! {
                fn beq(rs1, rs2, b_imm) {
                    self.tick_branch(rs1, rs2, b_imm, |lhs, rhs| lhs == rhs);
                }
            },

            (0b1100011, 0b001, _) => op! {
                fn bne(rs1, rs2, b_imm) {
                    self.tick_branch(rs1, rs2, b_imm, |lhs, rhs| lhs != rhs);
                }
            },

            (0b1100011, 0b100, _) => op! {
                fn blt(rs1, rs2, b_imm) {
                    self.tick_branch(rs1, rs2, b_imm, |lhs, rhs| lhs < rhs);
                }
            },

            (0b1100011, 0b110, _) => op! {
                fn bltu(rs1, rs2, b_imm) {
                    self.tick_branch(rs1, rs2, b_imm, |lhs, rhs| {
                        (lhs as u32) < (rhs as u32)
                    });
                }
            },

            (0b1100011, 0b101, _) => op! {
                fn bge(rs1, rs2, b_imm) {
                    self.tick_branch(rs1, rs2, b_imm, |lhs, rhs| lhs >= rhs);
                }
            },

            (0b1100011, 0b111, _) => op! {
                fn bgeu(rs1, rs2, b_imm) {
                    self.tick_branch(rs1, rs2, b_imm, |lhs, rhs| {
                        (lhs as u32) >= (rhs as u32)
                    });
                }
            },

            (0b1101111, _, _) => op! {
                fn jal(rd, j_imm) {
                    self.reg_store(rd, self.pc as i32);

                    self.pc =
                        self.pc.wrapping_add_signed(j_imm as i32).wrapping_sub(4);
                }
            },

            (0b1100111, 0b000, _) => op! {
                fn jalr(rd, rs1, i_imm) {
                    let rs1_value = self.regs[rs1];

                    self.reg_store(rd, self.pc as i32);
                    self.pc = rs1_value.wrapping_add(i_imm) as u32;
                }
            },

            (0b1110011, 0b000, _) => {
                let i_imm = op!(@arg i_imm);

                match i_imm {
                    0x01 => {
                        return Err(TickError::GotEbreak);
                    }
                    _ => {
                        return Err(TickError::UnknownInstruction { word });
                    }
                }
            }

            _ => {
                return Err(TickError::UnknownInstruction { word });
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn tick_branch(
        &mut self,
        rs1: usize,
        rs2: usize,
        imm: i32,
        op: fn(i32, i32) -> bool,
    ) {
        let lhs = self.regs[rs1];
        let rhs = self.regs[rs2];

        if op(lhs, rhs) {
            self.pc = self.pc.wrapping_add_signed(imm).wrapping_sub(4);
        }
    }

    #[inline(always)]
    fn tick_atomic(
        &mut self,
        rd: usize,
        rs1: usize,
        rs2: usize,
        op: Atomic,
    ) -> TickResult<()> {
        let prev =
            self.mem_atomic(self.regs[rs1] as u32, self.regs[rs2] as u32, op)?;

        self.reg_store(rd, prev as i32);

        Ok(())
    }

    #[inline(always)]
    fn mem_load<M>(
        &mut self,
        mmio: Option<M>,
        addr: u32,
        size: u8,
    ) -> TickResult<i32>
    where
        M: Mmio,
    {
        match Region::new(addr, size)? {
            Region::Mmio => mmio
                .ok_or(TickError::InvalidAccess { addr, size })?
                .load(addr),

            Region::Ram => self.ram_load(addr, size),
        }
        .map(|value| value as i32)
        .map_err(|_| TickError::InvalidAccess { addr, size })
    }

    #[inline(always)]
    fn mem_store<M>(
        &mut self,
        mmio: Option<M>,
        addr: u32,
        size: u8,
        value: u32,
    ) -> TickResult<()>
    where
        M: Mmio,
    {
        match Region::new(addr, size)? {
            Region::Mmio => mmio
                .ok_or(TickError::InvalidAccess { addr, size })?
                .store(addr, value),

            Region::Ram => self.ram_store(addr, size, value),
        }
        .map_err(|_| TickError::InvalidAccess { addr, size })
    }

    #[inline(always)]
    fn mem_atomic(
        &mut self,
        addr: u32,
        rhs: u32,
        op: Atomic,
    ) -> TickResult<u32> {
        let size = 4;

        if let Region::Mmio = Region::new(addr, size)? {
            return Err(TickError::InvalidAccess { addr, size });
        }

        let lhs = self
            .ram_load(addr, size)
            .map_err(|_| TickError::InvalidAccess { addr, size })?;

        let value = match op {
            Atomic::Add => lhs.wrapping_add(rhs),
            Atomic::Swap => rhs,
            Atomic::Xor => lhs ^ rhs,
            Atomic::And => lhs & rhs,
            Atomic::Or => lhs | rhs,
            Atomic::Min => (lhs as i32).min(rhs as i32) as u32,
            Atomic::Max => (lhs as i32).max(rhs as i32) as u32,
            Atomic::MinU => lhs.min(rhs),
            Atomic::MaxU => lhs.max(rhs),
        };

        self.ram_store(addr, size, value)
            .map_err(|_| TickError::InvalidAccess { addr, size })?;

        Ok(lhs)
    }

    #[inline(always)]
    fn ram_load(&self, addr: u32, size: u8) -> TickResult<u32, ()> {
        let addr = addr - Self::RAM_BASE;

        if addr as usize + size as usize > self.ram.len() {
            return Err(());
        }

        let mut value = 0;

        for offset in 0..size as usize {
            value |= (self.ram[addr as usize + offset] as u32) << (offset * 8);
        }

        Ok(value)
    }

    #[inline(always)]
    fn ram_store(
        &mut self,
        addr: u32,
        size: u8,
        value: u32,
    ) -> TickResult<(), ()> {
        let addr = addr - Self::RAM_BASE;

        if addr as usize + size as usize > self.ram.len() {
            return Err(());
        }

        for offset in 0..size as usize {
            self.ram[addr as usize + offset] = (value >> (offset * 8)) as u8;
        }

        Ok(())
    }

    #[inline(always)]
    fn reg_store(&mut self, id: usize, val: i32) {
        if id != 0 {
            self.regs[id] = val;
        }
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cpu")
            .field("pc", &self.pc)
            .field("regs", &self.regs)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Region {
    Mmio,
    Ram,
}

impl Region {
    #[inline(always)]
    fn new(addr: u32, size: u8) -> TickResult<Self> {
        if addr >= Cpu::MMIO_BASE {
            if size == 4 && addr % 4 == 0 {
                Ok(Region::Mmio)
            } else {
                Err(TickError::InvalidAccess { addr, size })
            }
        } else if addr >= Cpu::RAM_BASE {
            Ok(Region::Ram)
        } else if addr == 0 {
            Err(TickError::NullPointerAccess { addr, size })
        } else {
            Err(TickError::InvalidAccess { addr, size })
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Atomic {
    Add,
    Swap,
    Xor,
    And,
    Or,
    Min,
    Max,
    MinU,
    MaxU,
}

pub trait Mmio {
    fn load(&mut self, addr: u32) -> Result<u32, ()>;
    fn store(&mut self, addr: u32, value: u32) -> Result<(), ()>;
}

impl Mmio for () {
    fn load(&mut self, _: u32) -> Result<u32, ()> {
        Err(())
    }

    fn store(&mut self, _: u32, _: u32) -> Result<(), ()> {
        Err(())
    }
}
