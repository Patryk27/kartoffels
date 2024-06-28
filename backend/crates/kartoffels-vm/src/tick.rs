use super::{Mmio, Runtime};
use anyhow::{anyhow, Context, Result};

macro_rules! op {
    ($self:ident, $word:ident, $( $tt:tt )*) => {
        #[cfg(test)]
        eprintln!(
            "    {} # word=0x{:08x} pc=0x{:04x}",
            format!($($tt)*),
            $word,
            $self.pc,
        );
    };
}

impl Runtime {
    pub(super) fn do_tick(&mut self, mmio: &mut impl Mmio) -> Result<bool> {
        let word = self
            .mem_load::<4>(mmio, self.pc)
            .context("cannot fetch instruction")? as u32;

        let op = word & 0x7f;
        let rd = ((word >> 7) & 0x1f) as usize;
        let rs1 = ((word >> 15) & 0x1f) as usize;
        let rs2 = ((word >> 20) & 0x1f) as usize;
        let funct3 = (word >> 12) & 0x7;
        let funct5 = word >> 27;
        let funct7 = word >> 25;

        let i_imm = (word as i32 as i64) >> 20;
        let u_imm = (word as i32 as i64) >> 12;

        let s_imm = {
            ((word & 0xfe000000) as i32 as i64 >> 20)
                | (((word >> 7) & 0x1f) as i64)
        };

        let b_imm = {
            (((word & 0x80000000) as i32 >> 19) as u32
                | ((word & 0x80) << 4)
                | ((word >> 20) & 0x7e0)
                | ((word >> 7) & 0x1e)) as i32
        };

        let j_imm = {
            (((word & 0x80000000) as i32 >> 11) as u32
                | (word & 0xff000)
                | ((word >> 9) & 0x800)
                | ((word >> 20) & 0x7fe)) as i32
        };

        self.pc += 4;

        match (op, funct3, funct7) {
            (0b0110111, _, _) => {
                op!(self, word, "lui x{rd}, {u_imm}");

                self.reg_store(rd, u_imm << 12);
            }

            (0b0010111, _, _) => {
                op!(self, word, "auipc x{rd}, {u_imm}");

                self.reg_store(rd, (self.pc as i64) - 4 + (u_imm << 12));
            }

            (0b0110011, 0b000, 0b0000000) => {
                op!(self, word, "add x{rd}, x{rs1}, x{rs2}");

                let lhs = self.regs[rs1];
                let rhs = self.regs[rs2];

                self.reg_store(rd, lhs.wrapping_add(rhs));
            }

            (0b0111011, 0b000, 0b0000000) => {
                op!(self, word, "addw x{rd}, x{rs1}, x{rs2}");

                let lhs = self.regs[rs1];
                let rhs = self.regs[rs2];

                self.reg_store(rd, lhs.wrapping_add(rhs) as i32 as i64);
            }

            (0b0010011, 0b000, _) => {
                op!(self, word, "addi x{rd}, x{rs1}, {i_imm}");

                let lhs = self.regs[rs1];
                let rhs = i_imm;

                self.reg_store(rd, lhs.wrapping_add(rhs));
            }

            (0b0011011, 0b000, _) => {
                op!(self, word, "addiw x{rd}, x{rs1}, {i_imm}");

                let lhs = self.regs[rs1];
                let rhs = i_imm;

                self.reg_store(rd, lhs.wrapping_add(rhs) as i32 as i64);
            }

            (0b0110011, 0b000, 0b0100000) => {
                op!(self, word, "sub x{rd}, x{rs1}, x{rs2}");

                let lhs = self.regs[rs1];
                let rhs = self.regs[rs2];

                self.reg_store(rd, lhs.wrapping_sub(rhs));
            }

            (0b0111011, 0b000, 0b0100000) => {
                op!(self, word, "subw x{rd}, x{rs1}, x{rs2}");

                let lhs = self.regs[rs1];
                let rhs = self.regs[rs2];

                self.reg_store(rd, lhs.wrapping_sub(rhs) as i32 as i64);
            }

            (0b0110011, 0b000, 0b0000001) => {
                op!(self, word, "mul x{rd}, x{rs1}, x{rs2}");

                let lhs = self.regs[rs1];
                let rhs = self.regs[rs2];

                self.reg_store(rd, lhs.wrapping_mul(rhs));
            }

            (0b0110011, 0b001, 0b0000001) => {
                op!(self, word, "mulh x{rd}, x{rs1}, x{rs2}");

                let lhs = self.regs[rs1] as i128;
                let rhs = self.regs[rs2] as i128;

                self.reg_store(rd, (lhs.wrapping_mul(rhs) >> 64) as i64);
            }

            (0b0110011, 0b011, 0b0000001) => {
                op!(self, word, "mulhu x{rd}, x{rs1}, x{rs2}");

                let lhs = self.regs[rs1] as u64 as u128;
                let rhs = self.regs[rs2] as u64 as u128;

                self.reg_store(rd, (lhs.wrapping_mul(rhs) >> 64) as i64);
            }

            (0b0110011, 0b100, 0b0000001) => {
                op!(self, word, "div x{rd}, x{rs1}, x{rs2}");

                let lhs = self.regs[rs1];
                let rhs = self.regs[rs2];

                self.reg_store(rd, lhs.checked_div(rhs).unwrap_or(-1));
            }

            (0b0110011, 0b101, 0b0000001) => {
                op!(self, word, "divu x{rd}, x{rs1}, x{rs2}");

                let lhs = self.regs[rs1] as u64;
                let rhs = self.regs[rs2] as u64;

                self.reg_store(
                    rd,
                    lhs.checked_div(rhs).unwrap_or(-1i64 as u64) as i64,
                );
            }

            (0b0110011, 0b110, 0b0000001) => {
                op!(self, word, "rem x{rd}, x{rs1}, x{rs2}");

                let lhs = self.regs[rs1];
                let rhs = self.regs[rs2];

                self.reg_store(rd, lhs.checked_rem(rhs).unwrap_or(-1));
            }

            (0b0110011, 0b111, 0b0000001) => {
                op!(self, word, "remu x{rd}, x{rs1}, x{rs2}");

                let lhs = self.regs[rs1] as u64;
                let rhs = self.regs[rs2] as u64;

                self.reg_store(
                    rd,
                    lhs.checked_rem(rhs).unwrap_or(-1i64 as u64) as i64,
                );
            }

            (0b0110011, 0b111, 0b0000000) => {
                op!(self, word, "and x{rd}, x{rs1}, x{rs2}");

                let lhs = self.regs[rs1];
                let rhs = self.regs[rs2];

                self.reg_store(rd, lhs & rhs);
            }

            (0b0010011, 0b111, _) => {
                op!(self, word, "andi x{rd}, x{rs1}, {i_imm}");

                let lhs = self.regs[rs1];
                let rhs = i_imm;

                self.reg_store(rd, lhs & rhs);
            }

            (0b0110011, 0b110, 0b0000000) => {
                op!(self, word, "or x{rd}, x{rs1}, x{rs2}");

                let lhs = self.regs[rs1];
                let rhs = self.regs[rs2];

                self.reg_store(rd, lhs | rhs);
            }

            (0b0010011, 0b110, _) => {
                op!(self, word, "ori x{rd}, x{rs1}, {i_imm}");

                let lhs = self.regs[rs1];
                let rhs = i_imm;

                self.reg_store(rd, lhs | rhs);
            }

            (0b0110011, 0b100, 0b0000000) => {
                op!(self, word, "xor x{rd}, x{rs1}, x{rs2}");

                let lhs = self.regs[rs1];
                let rhs = self.regs[rs2];

                self.reg_store(rd, lhs ^ rhs);
            }

            (0b0010011, 0b100, _) => {
                op!(self, word, "xori x{rd}, x{rs1}, {i_imm}");

                let lhs = self.regs[rs1];
                let rhs = i_imm;

                self.reg_store(rd, lhs ^ rhs);
            }

            (0b0110011, 0b001, 0b0000000) => {
                op!(self, word, "sll x{rd}, x{rs1}, x{rs2}");

                let lhs = self.regs[rs1] as u64;
                let rhs = self.regs[rs2] as u32;

                self.reg_store(rd, lhs.wrapping_shl(rhs) as i64);
            }

            (0b0111011, 0b001, 0b0000000) => {
                op!(self, word, "sllw x{rd}, x{rs1}, x{rs2}");

                let lhs = self.regs[rs1] as u32;
                let rhs = self.regs[rs2] as u32;

                self.reg_store(rd, lhs.wrapping_shl(rhs) as i32 as i64);
            }

            (0b0010011, 0b001, _) if (i_imm >> 6) == 0x00 => {
                op!(self, word, "slli x{rd}, x{rs1}, {i_imm}");

                let lhs = self.regs[rs1] as u64;
                let rhs = i_imm as u32;

                self.reg_store(rd, lhs.wrapping_shl(rhs) as i64);
            }

            (0b0110011, 0b101, 0b0000000) => {
                op!(self, word, "srl x{rd}, x{rs1}, x{rs2}");

                let lhs = self.regs[rs1] as u64;
                let rhs = self.regs[rs2] as u32;

                self.reg_store(rd, lhs.wrapping_shr(rhs) as i64);
            }

            (0b0111011, 0b101, 0b0000000) => {
                op!(self, word, "srlw x{rd}, x{rs1}, x{rs2}");

                let lhs = self.regs[rs1] as u32;
                let rhs = self.regs[rs2] as u32;

                self.reg_store(rd, lhs.wrapping_shr(rhs) as i32 as i64);
            }

            (0b0010011, 0b101, _) if (i_imm >> 6) == 0x00 => {
                op!(self, word, "srli x{rd}, x{rs1}, {i_imm}");

                let lhs = self.regs[rs1] as u64;
                let rhs = i_imm as u32;

                self.reg_store(rd, lhs.wrapping_shr(rhs) as i64);
            }

            (0b0110011, 0b101, 0b0100000) => {
                op!(self, word, "sra x{rd}, x{rs1}, x{rs2}");

                let lhs = self.regs[rs1];
                let rhs = self.regs[rs2] as u32;

                self.reg_store(rd, lhs.wrapping_shr(rhs));
            }

            (0b0010011, 0b101, _) if (i_imm >> 5) == 0x20 => {
                let lhs = self.regs[rs1];
                let rhs = (i_imm as u32) & 0x3f;

                op!(self, word, "srai x{rd}, x{rs1}, {rhs}");

                self.reg_store(rd, lhs.wrapping_shr(rhs));
            }

            (0b0110011, 0b010, 0b0000000) => {
                op!(self, word, "slt x{rd}, x{rs1}, x{rs2}");

                let lhs = self.regs[rs1];
                let rhs = self.regs[rs2];

                self.reg_store(rd, (lhs < rhs) as i64);
            }

            (0b0010011, 0b010, _) => {
                op!(self, word, "slti x{rd}, x{rs1}, {i_imm}");

                let lhs = self.regs[rs1];
                let rhs = i_imm;

                self.reg_store(rd, (lhs < rhs) as i64);
            }

            (0b0110011, 0b011, 0b0000000) => {
                op!(self, word, "sltu x{rd}, x{rs1}, x{rs2}");

                let lhs = self.regs[rs1] as u64;
                let rhs = self.regs[rs2] as u64;

                self.reg_store(rd, (lhs < rhs) as i64);
            }

            (0b0010011, 0b011, _) => {
                op!(self, word, "sltiu x{rd}, x{rs1}, {i_imm}");

                let lhs = self.regs[rs1] as u64;
                let rhs = i_imm as u64;

                self.reg_store(rd, (lhs < rhs) as i64);
            }

            (0b0000011, 0b000, _) => {
                op!(self, word, "lb x{rd}, x{rs1}, {i_imm}");

                let addr = (self.regs[rs1] + i_imm) as u64;
                let val = self.mem_load::<1>(mmio, addr)? as i8 as i64;

                self.reg_store(rd, val);
            }

            (0b0000011, 0b100, _) => {
                op!(self, word, "lbu x{rd}, x{rs1}, x{rs2}");

                let addr = (self.regs[rs1] + i_imm) as u64;
                let val = self.mem_load::<1>(mmio, addr)?;

                self.reg_store(rd, val);
            }

            (0b0000011, 0b001, _) => {
                op!(self, word, "lh x{rd}, x{rs1}, {i_imm}");

                let addr = (self.regs[rs1] + i_imm) as u64;
                let val = self.mem_load::<2>(mmio, addr)? as i16 as i64;

                self.reg_store(rd, val);
            }

            (0b0000011, 0b101, _) => {
                op!(self, word, "lhu x{rd}, x{rs1}, x{rs2}");

                let addr = (self.regs[rs1] + i_imm) as u64;
                let val = self.mem_load::<2>(mmio, addr)?;

                self.reg_store(rd, val);
            }

            (0b0000011, 0b010, _) => {
                op!(self, word, "lw x{rd}, x{rs1}, x{rs2}");

                let addr = (self.regs[rs1] + i_imm) as u64;
                let val = self.mem_load::<4>(mmio, addr)? as i32 as i64;

                self.reg_store(rd, val);
            }

            (0b0000011, 0b110, _) => {
                op!(self, word, "lwu x{rd}, x{rs1}, x{rs2}");

                let addr = (self.regs[rs1] + i_imm) as u64;
                let val = self.mem_load::<4>(mmio, addr)?;

                self.reg_store(rd, val);
            }

            (0b0000011, 0b011, _) => {
                op!(self, word, "ld x{rd}, x{rs1}, x{rs2}");

                let addr = (self.regs[rs1] + i_imm) as u64;
                let val = self.mem_load::<8>(mmio, addr)?;

                self.reg_store(rd, val);
            }

            (0b0100011, 0b000, _) => {
                op!(self, word, "sb x{rd}, x{rs1}, x{rs2}");

                self.mem_store::<1>(
                    mmio,
                    self.regs[rs1].wrapping_add(s_imm) as u64,
                    self.regs[rs2],
                )?;
            }

            (0b0100011, 0b001, _) => {
                op!(self, word, "sh x{rd}, x{rs1}, x{rs2}");

                self.mem_store::<2>(
                    mmio,
                    self.regs[rs1].wrapping_add(s_imm) as u64,
                    self.regs[rs2],
                )?;
            }

            (0b0100011, 0b010, _) => {
                op!(self, word, "sw x{rd}, x{rs1}, x{rs2}");

                self.mem_store::<4>(
                    mmio,
                    self.regs[rs1].wrapping_add(s_imm) as u64,
                    self.regs[rs2],
                )?;
            }

            (0b0100011, 0b011, _) => {
                op!(self, word, "sd x{rd}, x{rs1}, x{rs2}");

                self.mem_store::<8>(
                    mmio,
                    self.regs[rs1].wrapping_add(s_imm) as u64,
                    self.regs[rs2],
                )?;
            }

            (0b0101111, 0b010, _) if funct5 == 0b00000 => {
                op!(self, word, "amoadd.w x{rd}, x{rs1}, x{rs2}");

                self.do_atomic::<4>(mmio, rd, rs1, rs2, |lhs, rhs| {
                    (lhs as i32).wrapping_add(rhs as i32) as i64
                })?;
            }

            (0b0101111, 0b010, _) if funct5 == 0b01100 => {
                op!(self, word, "amoand.w x{rd}, x{rs1}, x{rs2}");

                self.do_atomic::<4>(mmio, rd, rs1, rs2, |lhs, rhs| lhs & rhs)?;
            }

            (0b0101111, 0b010, _) if funct5 == 0b01000 => {
                op!(self, word, "amoor.w x{rd}, x{rs1}, x{rs2}");

                self.do_atomic::<4>(mmio, rd, rs1, rs2, |lhs, rhs| lhs | rhs)?;
            }

            (0b0001111, 0b000, _) => {
                op!(self, word, "fence");
            }

            (0b1100011, 0b000, _) => {
                op!(self, word, "beq x{rs1}, x{rs2}, {b_imm}");

                self.do_branch(rs1, rs2, b_imm, |lhs, rhs| lhs == rhs);
            }

            (0b1100011, 0b001, _) => {
                op!(self, word, "bne x{rs1}, x{rs2}, {b_imm}");

                self.do_branch(rs1, rs2, b_imm, |lhs, rhs| lhs != rhs);
            }

            (0b1100011, 0b100, _) => {
                op!(self, word, "blt x{rs1}, x{rs2}, {b_imm}");

                self.do_branch(rs1, rs2, b_imm, |lhs, rhs| lhs < rhs);
            }

            (0b1100011, 0b110, _) => {
                op!(self, word, "bltu x{rs1}, x{rs2}, {b_imm}");

                self.do_branch(rs1, rs2, b_imm, |lhs, rhs| {
                    (lhs as u64) < (rhs as u64)
                });
            }

            (0b1100011, 0b101, _) => {
                op!(self, word, "bge x{rs1}, x{rs2}, {b_imm}");

                self.do_branch(rs1, rs2, b_imm, |lhs, rhs| lhs >= rhs);
            }

            (0b1100011, 0b111, _) => {
                op!(self, word, "bgeu x{rs1}, x{rs2}, {b_imm}");

                self.do_branch(rs1, rs2, b_imm, |lhs, rhs| {
                    (lhs as u64) >= (rhs as u64)
                });
            }

            (0b1101111, _, _) => {
                op!(self, word, "jal {j_imm}");

                #[cfg(test)]
                if j_imm == 0 {
                    return Err(anyhow!("infinite loop detected"));
                }

                self.reg_store(rd, self.pc as i64);

                self.pc =
                    self.pc.wrapping_add_signed(j_imm as i64).wrapping_sub(4);
            }

            (0b1100111, 0b000, _) => {
                op!(self, word, "jalr x{rs1}, {i_imm}");

                let rs1_val = self.regs[rs1];

                self.reg_store(rd, self.pc as i64);
                self.pc = rs1_val.wrapping_add(i_imm) as u64;
            }

            (0b1110011, 0b000, _) if i_imm == 1 => {
                op!(self, word, "ebreak");

                return Ok(false);
            }

            _ => {
                #[cfg(test)]
                eprintln!(
                    "0x{:04x}: 0x{:08x} ({:032b} - {:07b}:{:03b}:{:07b})",
                    self.pc, word, word, op, funct3, funct7
                );

                return Err(anyhow!("unknown instruction: 0x{:08x}", word));
            }
        }

        Ok(true)
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

    fn do_atomic<const BYTES: usize>(
        &mut self,
        mmio: &mut impl Mmio,
        rd: usize,
        rs1: usize,
        rs2: usize,
        op: fn(i64, i64) -> i64,
    ) -> Result<()> {
        let addr = self.regs[rs1] as u64;
        let old_val = self.mem_load::<BYTES>(mmio, addr)?;
        let new_val = op(old_val, self.regs[rs2]);

        self.mem_store::<BYTES>(mmio, addr, new_val)?;
        self.reg_store(rd, old_val);

        Ok(())
    }

    fn reg_store(&mut self, id: usize, val: i64) {
        if id != 0 {
            self.regs[id] = val;
        }
    }
}
