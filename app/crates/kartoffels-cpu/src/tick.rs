use super::{Cpu, Mmio};
use std::cmp;
use std::ops::{BitAnd, BitOr, BitXor};

impl Cpu {
    pub(super) fn do_tick(&mut self, mmio: impl Mmio) -> Result<(), Box<str>> {
        let word = self.mem_load::<(), 4>(None, self.pc)? as u32;

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
                (word as i32 as i64) >> 20
            };

            (@arg u_imm) => {
                (word as i32 as i64) >> 12
            };

            (@arg s_imm) => {
                ((word & 0xfe000000) as i32 as i64 >> 20)
                    | (((word >> 7) & 0x1f) as i64)
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

        macro_rules! unknown_instr {
            () => {
                format!("unknown instruction: 0x{word:08x}").into()
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
                    self.reg_store(rd, (self.pc as i64) - 4 + (u_imm << 12));
                }
            },

            (0b0110011, 0b000, 0b0000000) => op! {
                fn add(rd, rs1, rs2) {
                    let lhs = self.regs[rs1];
                    let rhs = self.regs[rs2];

                    self.reg_store(rd, lhs.wrapping_add(rhs));
                }
            },

            (0b0111011, 0b000, 0b0000000) => op! {
                fn addw(rd, rs1, rs2) {
                    let lhs = self.regs[rs1];
                    let rhs = self.regs[rs2];

                    self.reg_store(rd, lhs.wrapping_add(rhs) as i32 as i64);
                }
            },

            (0b0010011, 0b000, _) => op! {
                fn addi(rd, rs1, i_imm) {
                    let lhs = self.regs[rs1];
                    let rhs = i_imm;

                    self.reg_store(rd, lhs.wrapping_add(rhs));
                }
            },

            (0b0011011, 0b000, _) => op! {
                fn addiw(rd, rs1, i_imm) {
                    let lhs = self.regs[rs1];
                    let rhs = i_imm;

                    self.reg_store(rd, lhs.wrapping_add(rhs) as i32 as i64);
                }
            },

            (0b0110011, 0b000, 0b0100000) => op! {
                fn sub(rd, rs1, rs2) {
                    let lhs = self.regs[rs1];
                    let rhs = self.regs[rs2];

                    self.reg_store(rd, lhs.wrapping_sub(rhs));
                }
            },

            (0b0111011, 0b000, 0b0100000) => op! {
                fn subw(rd, rs1, rs2) {
                    let lhs = self.regs[rs1];
                    let rhs = self.regs[rs2];

                    self.reg_store(rd, lhs.wrapping_sub(rhs) as i32 as i64);
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
                    let lhs = self.regs[rs1] as i128;
                    let rhs = self.regs[rs2] as i128;

                    self.reg_store(rd, (lhs.wrapping_mul(rhs) >> 64) as i64);
                }
            },

            (0b0110011, 0b010, 0b0000001) => op! {
                fn mulhsu(rd, rs1, rs2) {
                    let lhs = self.regs[rs1] as i128 as u128;
                    let rhs = self.regs[rs2] as u64 as u128;

                    self.reg_store(rd, (lhs.wrapping_mul(rhs) >> 64) as u64 as i64);
                }
            },

            (0b0110011, 0b011, 0b0000001) => op! {
                fn mulhu(rd, rs1, rs2) {
                    let lhs = self.regs[rs1] as u64 as u128;
                    let rhs = self.regs[rs2] as u64 as u128;

                    self.reg_store(rd, (lhs.wrapping_mul(rhs) >> 64) as i64);
                }
            },

            (0b0111011, 0b000, 0b0000001) => op! {
                fn mulw(rd, rs1, rs2) {
                    let lhs = self.regs[rs1];
                    let rhs = self.regs[rs2];

                    self.reg_store(rd, lhs.wrapping_mul(rhs) as i32 as i64);
                }
            },

            (0b0110011, 0b100, 0b0000001) => op! {
                fn div(rd, rs1, rs2) {
                    let lhs = self.regs[rs1];
                    let rhs = self.regs[rs2];

                    self.reg_store(rd, lhs.checked_div(rhs).unwrap_or(-1));
                }
            },

            (0b0111011, 0b100, 0b0000001) => op! {
                fn divw(rd, rs1, rs2) {
                    let lhs = self.regs[rs1] as i32;
                    let rhs = self.regs[rs2] as i32;

                    self.reg_store(rd, lhs.checked_div(rhs).unwrap_or(-1) as i64);
                }
            },

            (0b0110011, 0b101, 0b0000001) => op! {
                fn divu(rd, rs1, rs2) {
                    let lhs = self.regs[rs1] as u64;
                    let rhs = self.regs[rs2] as u64;

                    self.reg_store(
                        rd,
                        lhs.checked_div(rhs).unwrap_or(-1i64 as u64) as i64,
                    );
                }
            },

            (0b0111011, 0b101, 0b0000001) => op! {
                fn divuv(rd, rs1, rs2) {
                    let lhs = self.regs[rs1] as u32;
                    let rhs = self.regs[rs2] as u32;

                    self.reg_store(
                        rd,
                        lhs.checked_div(rhs)
                            .map(|val| val as i32 as i64)
                            .unwrap_or(-1),
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

            (0b0111011, 0b110, 0b0000001) => op! {
                fn remw(rd, rs1, rs2) {
                    let lhs = self.regs[rs1] as i32;
                    let rhs = self.regs[rs2] as i32;

                    self.reg_store(rd, lhs.checked_rem(rhs).unwrap_or(-1) as i64);
                }
            },

            (0b0110011, 0b111, 0b0000001) => op! {
                fn remu(rd, rs1, rs2) {
                    let lhs = self.regs[rs1] as u64;
                    let rhs = self.regs[rs2] as u64;

                    self.reg_store(
                        rd,
                        lhs.checked_rem(rhs).unwrap_or(-1i64 as u64) as i64,
                    );
                }
            },

            (0b0111011, 0b111, 0b0000001) => op! {
                fn remuw(rd, rs1, rs2) {
                    let lhs = self.regs[rs1] as u32;
                    let rhs = self.regs[rs2] as u32;

                    self.reg_store(
                        rd,
                        lhs.checked_rem(rhs)
                            .map(|val| val as i32 as i64)
                            .unwrap_or(-1),
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
                    let lhs = self.regs[rs1] as u64;
                    let rhs = self.regs[rs2] as u32;

                    self.reg_store(rd, lhs.wrapping_shl(rhs) as i64);
                }
            },

            (0b0111011, 0b001, 0b0000000) => op! {
                fn sllw(rd, rs1, rs2) {
                    let lhs = self.regs[rs1] as u32;
                    let rhs = self.regs[rs2] as u32;

                    self.reg_store(rd, lhs.wrapping_shl(rhs) as i32 as i64);
                }
            },

            (0b0010011, 0b001, _) => {
                let i_imm = op!(@arg i_imm);

                match i_imm >> 6 {
                    0x00 => op! {
                        fn slli(rd, rs1, i_imm) {
                            let lhs = self.regs[rs1] as u64;
                            let rhs = i_imm as u32;

                            self.reg_store(rd, lhs.wrapping_shl(rhs) as i64);
                        }
                    },

                    _ => {
                        return Err(unknown_instr!());
                    }
                }
            }

            (0b0011011, 0b001, _) => {
                let i_imm = op!(@arg i_imm);

                match i_imm >> 6 {
                    0x00 => op! {
                        fn slliw(rd, rs1, i_imm) {
                            let lhs = self.regs[rs1] as u64;
                            let rhs = i_imm as u32;

                            self.reg_store(rd, lhs.wrapping_shl(rhs) as i32 as i64);
                        }
                    },

                    _ => {
                        return Err(unknown_instr!());
                    }
                }
            }

            (0b0110011, 0b101, 0b0000000) => op! {
                fn srl(rd, rs1, rs2) {
                    let lhs = self.regs[rs1] as u64;
                    let rhs = self.regs[rs2] as u32;

                    self.reg_store(rd, lhs.wrapping_shr(rhs) as i64);
                }
            },

            (0b0111011, 0b101, 0b0000000) => op! {
                fn srlw(rd, rs1, rs2) {
                    let lhs = self.regs[rs1] as u32;
                    let rhs = self.regs[rs2] as u32;

                    self.reg_store(rd, lhs.wrapping_shr(rhs) as i32 as i64);
                }
            },

            (0b0010011, 0b101, _) => {
                let i_imm = op!(@arg i_imm);

                match i_imm >> 6 {
                    0x00 => op! {
                        fn srli(rd, rs1, i_imm) {
                            let lhs = self.regs[rs1] as u64;
                            let rhs = i_imm as u32;

                            self.reg_store(rd, lhs.wrapping_shr(rhs) as i64);
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
                        return Err(unknown_instr!());
                    }
                }
            }

            (0b0011011, 0b101, _) => {
                let i_imm = op!(@arg i_imm);

                match i_imm >> 6 {
                    0x00 => op! {
                        fn srliw(rd, rs1, i_imm) {
                            let lhs = self.regs[rs1] as u32;
                            let rhs = i_imm as u32;

                            self.reg_store(rd, lhs.wrapping_shr(rhs) as i32 as i64);
                        }
                    },

                    0x10 => op! {
                        fn sraiw(rd, rs1, i_imm) {
                            let lhs = self.regs[rs1] as i32;
                            let rhs = (i_imm as u32) & 0x3f;

                            self.reg_store(rd, lhs.wrapping_shr(rhs) as i64);
                        }
                    },

                    _ => {
                        return Err(unknown_instr!());
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

            (0b0111011, 0b101, 0b0100000) => op! {
                fn sraw(rd, rs1, rs2) {
                    let lhs = self.regs[rs1] as i32;
                    let rhs = self.regs[rs2] as u32;

                    self.reg_store(rd, lhs.wrapping_shr(rhs) as i64);
                }
            },

            (0b0110011, 0b010, 0b0000000) => op! {
                fn slt(rd, rs1, rs2) {
                    let lhs = self.regs[rs1];
                    let rhs = self.regs[rs2];

                    self.reg_store(rd, (lhs < rhs) as i64);
                }
            },

            (0b0010011, 0b010, _) => op! {
                fn slti(rd, rs1, i_imm) {
                    let lhs = self.regs[rs1];
                    let rhs = i_imm;

                    self.reg_store(rd, (lhs < rhs) as i64);
                }
            },

            (0b0110011, 0b011, 0b0000000) => op! {
                fn sltu(rd, rs1, rs2) {
                    let lhs = self.regs[rs1] as u64;
                    let rhs = self.regs[rs2] as u64;

                    self.reg_store(rd, (lhs < rhs) as i64);
                }
            },

            (0b0010011, 0b011, _) => op! {
                fn sltiu(rd, rs1, i_imm) {
                    let lhs = self.regs[rs1] as u64;
                    let rhs = i_imm as u64;

                    self.reg_store(rd, (lhs < rhs) as i64);
                }
            },

            (0b0000011, 0b000, _) => op! {
                fn lb(rd, rs1, i_imm) {
                    let addr = (self.regs[rs1] + i_imm) as u64;
                    let val = self.mem_load::<_, 1>(Some(mmio), addr)? as i8 as i64;

                    self.reg_store(rd, val);
                }
            },

            (0b0000011, 0b100, _) => op! {
                fn lbu(rd, rs1, i_imm) {
                    let addr = (self.regs[rs1] + i_imm) as u64;
                    let val = self.mem_load::<_, 1>(Some(mmio), addr)?;

                    self.reg_store(rd, val);
                }
            },

            (0b0000011, 0b001, _) => op! {
                fn lh(rd, rs1, i_imm) {
                    let addr = (self.regs[rs1] + i_imm) as u64;
                    let val = self.mem_load::<_, 2>(Some(mmio), addr)? as i16 as i64;

                    self.reg_store(rd, val);
                }
            },

            (0b0000011, 0b101, _) => op! {
                fn lhu(rd, rs1, i_imm) {
                    let addr = (self.regs[rs1] + i_imm) as u64;
                    let val = self.mem_load::<_, 2>(Some(mmio), addr)?;

                    self.reg_store(rd, val);
                }
            },

            (0b0000011, 0b010, _) => op! {
                fn lw(rd, rs1, i_imm) {
                    let addr = (self.regs[rs1] + i_imm) as u64;
                    let val = self.mem_load::<_, 4>(Some(mmio), addr)? as i32 as i64;

                    self.reg_store(rd, val);
                }
            },

            (0b0000011, 0b110, _) => op! {
                fn lwu(rd, rs1, i_imm) {
                    let addr = (self.regs[rs1] + i_imm) as u64;
                    let val = self.mem_load::<_, 4>(Some(mmio), addr)?;

                    self.reg_store(rd, val);
                }
            },

            (0b0000011, 0b011, _) => op! {
                fn ld(rd, rs1, i_imm) {
                    let addr = (self.regs[rs1] + i_imm) as u64;
                    let val = self.mem_load::<_, 8>(Some(mmio), addr)?;

                    self.reg_store(rd, val);
                }
            },

            (0b0100011, 0b000, _) => op! {
                fn sb(rs1, rs2, s_imm) {
                    let addr = self.regs[rs1].wrapping_add(s_imm) as u64;
                    let val = self.regs[rs2];

                    self.mem_store::<_, 1>(Some(mmio), addr, val)?;
                }
            },

            (0b0100011, 0b001, _) => op! {
                fn sh(rs1, rs2, s_imm) {
                    let addr = self.regs[rs1].wrapping_add(s_imm) as u64;
                    let val = self.regs[rs2];

                    self.mem_store::<_, 2>(Some(mmio), addr, val)?;
                }
            },

            (0b0100011, 0b010, _) => op! {
                fn sw(rs1, rs2, s_imm) {
                    let addr = self.regs[rs1].wrapping_add(s_imm) as u64;
                    let val = self.regs[rs2];

                    self.mem_store::<_, 4>(Some(mmio), addr, val)?;
                }
            },

            (0b0100011, 0b011, _) => op! {
                fn sd(rs1, rs2, s_imm) {
                    let addr = self.regs[rs1].wrapping_add(s_imm) as u64;
                    let val = self.regs[rs2];

                    self.mem_store::<_, 8>(Some(mmio), addr, val)?;
                }
            },

            (0b0101111, 0b010, _) => {
                // funct7's low bits encode the ordering semantics (acquire
                // and/or release) which we don't care about
                let funct5 = funct7 >> 2;

                match funct5 {
                    0b00000 => op! {
                        fn amoaddw(rd, rs1, rs2) {
                            self.do_atomic::<4>(rd, rs1, rs2, |lhs, rhs| {
                                (lhs as i32).wrapping_add(rhs as i32) as i64
                            })?;
                        }
                    },

                    0b00001 => op! {
                        fn amoswapw(rd, rs1, rs2) {
                            self.do_atomic::<4>(rd, rs1, rs2, |_, rhs| rhs)?;
                        }
                    },

                    0b00010 => op! {
                        fn lrw(rd, rs1) {
                            let addr = self.regs[rs1] as u64;
                            let val = self.mem_load::<(), 4>(None, addr)? as i32 as i64;

                            self.reg_store(rd, val);
                        }
                    },

                    0b00011 => op! {
                        fn scw(rd, rs1, rs2) {
                            let addr = self.regs[rs1] as u64;
                            let val = self.regs[rs2];

                            self.mem_store::<(), 4>(None, addr, val)?;
                            self.regs[rd] = 0;
                        }
                    },

                    0b00100 => op! {
                        fn amoxorw(rd, rs1, rs2) {
                            self.do_atomic::<4>(rd, rs1, rs2, BitXor::bitxor)?;
                        }
                    },

                    0b01100 => op! {
                        fn amoandw(rd, rs1, rs2) {
                            self.do_atomic::<4>(rd, rs1, rs2, BitAnd::bitand)?;
                        }
                    },

                    0b01000 => op! {
                        fn amoorw(rd, rs1, rs2) {
                            self.do_atomic::<4>(rd, rs1, rs2, BitOr::bitor)?;
                        }
                    },

                    0b10000 => op! {
                        fn amominw(rd, rs1, rs2) {
                            self.do_atomic::<4>(rd, rs1, rs2, cmp::min)?;
                        }
                    },

                    0b10100 => op! {
                        fn amomaxw(rd, rs1, rs2) {
                            self.do_atomic::<4>(rd, rs1, rs2, cmp::max)?;
                        }
                    },

                    0b11000 => op! {
                        fn amominuw(rd, rs1, rs2) {
                            self.do_atomic::<4>(rd, rs1, rs2, |lhs, rhs| {
                                cmp::min(lhs as u32, rhs as u32) as u64 as i64
                            })?;
                        }
                    },

                    0b11100 => op! {
                        fn amomaxuw(rd, rs1, rs2) {
                            self.do_atomic::<4>(rd, rs1, rs2, |lhs, rhs| {
                                cmp::max(lhs as u32, rhs as u32) as u64 as i64
                            })?;
                        }
                    },

                    _ => {
                        return Err(unknown_instr!());
                    }
                }
            }

            (0b0101111, 0b011, _) => {
                // funct7's low bits encode the ordering semantics (acquire
                // and/or release) which we don't care about
                let funct5 = funct7 >> 2;

                match funct5 {
                    0b00000 => op! {
                        fn amoaddw(rd, rs1, rs2) {
                            self.do_atomic::<8>(rd, rs1, rs2, i64::wrapping_add)?;
                        }
                    },

                    0b00001 => op! {
                        fn amoswapd(rd, rs1, rs2) {
                            self.do_atomic::<8>(rd, rs1, rs2, |_, rhs| rhs)?;
                        }
                    },

                    0b00010 => op! {
                        fn lrd(rd, rs1) {
                            let addr = self.regs[rs1] as u64;
                            let val = self.mem_load::<(), 8>(None, addr)?;

                            self.reg_store(rd, val);
                        }
                    },

                    0b00011 => op! {
                        fn scd(rd, rs1, rs2) {
                            let addr = self.regs[rs1] as u64;
                            let val = self.regs[rs2];

                            self.mem_store::<(), 8>(None, addr, val)?;
                            self.regs[rd] = 0;
                        }
                    },

                    0b00100 => op! {
                        fn amoxord(rd, rs1, rs2) {
                            self.do_atomic::<8>(rd, rs1, rs2, BitXor::bitxor)?;
                        }
                    },

                    0b01100 => op! {
                        fn amoandd(rd, rs1, rs2) {
                            self.do_atomic::<8>(rd, rs1, rs2, BitAnd::bitand)?;
                        }
                    },

                    0b01000 => op! {
                        fn amoord(rd, rs1, rs2) {
                            self.do_atomic::<8>(rd, rs1, rs2, BitOr::bitor)?;
                        }
                    },

                    0b10000 => op! {
                        fn amomind(rd, rs1, rs2) {
                            self.do_atomic::<8>(rd, rs1, rs2, cmp::min)?;
                        }
                    },

                    0b10100 => op! {
                        fn amomaxd(rd, rs1, rs2) {
                            self.do_atomic::<8>(rd, rs1, rs2, cmp::max)?;
                        }
                    },

                    0b11000 => op! {
                        fn amominud(rd, rs1, rs2) {
                            self.do_atomic::<8>(rd, rs1, rs2, |lhs, rhs| {
                                cmp::min(lhs as u64, rhs as u64) as i64
                            })?;
                        }
                    },

                    0b11100 => op! {
                        fn amomaxud(rd, rs1, rs2) {
                            self.do_atomic::<8>(rd, rs1, rs2, |lhs, rhs| {
                                cmp::max(lhs as u64, rhs as u64) as i64
                            })?;
                        }
                    },

                    _ => {
                        return Err(unknown_instr!());
                    }
                }
            }

            (0b0001111, 0b000, _) => {
                // atomic fence
            }

            (0b1100011, 0b000, _) => op! {
                fn beq(rs1, rs2, b_imm) {
                    self.do_branch(rs1, rs2, b_imm, |lhs, rhs| lhs == rhs);
                }
            },

            (0b1100011, 0b001, _) => op! {
                fn bne(rs1, rs2, b_imm) {
                    self.do_branch(rs1, rs2, b_imm, |lhs, rhs| lhs != rhs);
                }
            },

            (0b1100011, 0b100, _) => op! {
                fn blt(rs1, rs2, b_imm) {
                    self.do_branch(rs1, rs2, b_imm, |lhs, rhs| lhs < rhs);
                }
            },

            (0b1100011, 0b110, _) => op! {
                fn bltu(rs1, rs2, b_imm) {
                    self.do_branch(rs1, rs2, b_imm, |lhs, rhs| {
                        (lhs as u64) < (rhs as u64)
                    });
                }
            },

            (0b1100011, 0b101, _) => op! {
                fn bge(rs1, rs2, b_imm) {
                    self.do_branch(rs1, rs2, b_imm, |lhs, rhs| lhs >= rhs);
                }
            },

            (0b1100011, 0b111, _) => op! {
                fn bgeu(rs1, rs2, b_imm) {
                    self.do_branch(rs1, rs2, b_imm, |lhs, rhs| {
                        (lhs as u64) >= (rhs as u64)
                    });
                }
            },

            (0b1101111, _, _) => op! {
                fn jal(rd, j_imm) {
                    #[cfg(test)]
                    if j_imm == 0 {
                        return Err("infinite loop detected".into());
                    }

                    self.reg_store(rd, self.pc as i64);

                    self.pc =
                        self.pc.wrapping_add_signed(j_imm as i64).wrapping_sub(4);
                }
            },

            (0b1100111, 0b000, _) => op! {
                fn jalr(rd, rs1, i_imm) {
                    let rs1_val = self.regs[rs1];

                    self.reg_store(rd, self.pc as i64);
                    self.pc = rs1_val.wrapping_add(i_imm) as u64;
                }
            },

            (0b1110011, 0b000, _) => {
                let i_imm = op!(@arg i_imm);

                match i_imm {
                    0x01 => {
                        return Err("got `ebreak`".into());
                    }

                    _ => {
                        return Err(unknown_instr!());
                    }
                }
            }

            _ => {
                return Err(unknown_instr!());
            }
        }

        Ok(())
    }

    fn do_branch(
        &mut self,
        rs1: usize,
        rs2: usize,
        imm: i32,
        op: fn(i64, i64) -> bool,
    ) {
        let lhs = self.regs[rs1];
        let rhs = self.regs[rs2];

        if op(lhs, rhs) {
            self.pc = self.pc.wrapping_add_signed(imm as i64).wrapping_sub(4);
        }
    }

    fn do_atomic<const SIZE: usize>(
        &mut self,
        rd: usize,
        rs1: usize,
        rs2: usize,
        op: fn(i64, i64) -> i64,
    ) -> Result<(), Box<str>> {
        let addr = self.regs[rs1] as u64;

        let old_val = self.mem_load::<(), SIZE>(None, addr)?;
        let new_val = op(old_val, self.regs[rs2]);

        self.mem_store::<(), SIZE>(None, addr, new_val)?;
        self.reg_store(rd, old_val);

        Ok(())
    }

    fn reg_store(&mut self, id: usize, val: i64) {
        if id != 0 {
            self.regs[id] = val;
        }
    }
}
