use super::{Mmio, Runtime};
use anyhow::{anyhow, Context, Result};

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

        #[cfg(test)]
        if false {
            eprintln!(
                "0x{:04x}: {:032b} (0x{:08x}; {:07b}:{:03b}:{:07b})",
                self.pc, word, word, op, funct3, funct7
            );
        }

        self.pc += 4;

        match (op, funct3, funct7) {
            // lui
            (0b0110111, _, _) => {
                self.reg_store(rd, u_imm << 12);
            }

            // auipc
            (0b0010111, _, _) => {
                self.reg_store(rd, (self.pc as i64) - 4 + (u_imm << 12));
            }

            // -----------------------------------------------------------------

            // add
            (0b0110011, 0b000, 0b0000000) => {
                let lhs = self.regs[rs1];
                let rhs = self.regs[rs2];

                self.reg_store(rd, lhs.wrapping_add(rhs));
            }

            // addw
            (0b0111011, 0b000, 0b0000000) => {
                let lhs = self.regs[rs1];
                let rhs = self.regs[rs2];

                self.reg_store(rd, lhs.wrapping_add(rhs) as i32 as i64);
            }

            // addi
            (0b0010011, 0b000, _) => {
                let lhs = self.regs[rs1];
                let rhs = i_imm;

                self.reg_store(rd, lhs.wrapping_add(rhs));
            }

            // addiw
            (0b0011011, 0b000, _) => {
                let lhs = self.regs[rs1];
                let rhs = i_imm;

                self.reg_store(rd, lhs.wrapping_add(rhs) as i32 as i64);
            }

            // sub
            (0b0110011, 0b000, 0b0100000) => {
                let lhs = self.regs[rs1];
                let rhs = self.regs[rs2];

                self.reg_store(rd, lhs.wrapping_sub(rhs));
            }

            // subw
            (0b0111011, 0b000, 0b0100000) => {
                let lhs = self.regs[rs1];
                let rhs = self.regs[rs2];

                self.reg_store(rd, lhs.wrapping_sub(rhs) as i32 as i64);
            }

            // mul
            (0b0110011, 0b000, 0b0000001) => {
                let lhs = self.regs[rs1];
                let rhs = self.regs[rs2];

                self.reg_store(rd, lhs.wrapping_mul(rhs));
            }

            // mulh
            (0b0110011, 0b001, 0b0000001) => {
                let lhs = self.regs[rs1] as i128;
                let rhs = self.regs[rs2] as i128;

                self.reg_store(rd, (lhs.wrapping_mul(rhs) >> 64) as i64);
            }

            // mulhu
            (0b0110011, 0b011, 0b0000001) => {
                let lhs = self.regs[rs1] as u128;
                let rhs = self.regs[rs2] as u128;

                self.reg_store(rd, (lhs.wrapping_mul(rhs) >> 64) as i64);
            }

            // TODO mulsu, mulu

            // div
            (0b0110011, 0b100, 0b0000001) => {
                let lhs = self.regs[rs1];
                let rhs = self.regs[rs2];

                self.reg_store(rd, lhs.checked_div(rhs).unwrap_or(-1));
            }

            // divu
            (0b0110011, 0b101, 0b0000001) => {
                let lhs = self.regs[rs1] as u64;
                let rhs = self.regs[rs2] as u64;

                self.reg_store(
                    rd,
                    lhs.checked_div(rhs).unwrap_or(-1i64 as u64) as i64,
                );
            }

            // rem
            (0b0110011, 0b110, 0b0000001) => {
                let lhs = self.regs[rs1];
                let rhs = self.regs[rs2];

                self.reg_store(rd, lhs.checked_rem(rhs).unwrap_or(-1));
            }

            // remu
            (0b0110011, 0b111, 0b0000001) => {
                let lhs = self.regs[rs1] as u64;
                let rhs = self.regs[rs2] as u64;

                self.reg_store(
                    rd,
                    lhs.checked_rem(rhs).unwrap_or(-1i64 as u64) as i64,
                );
            }

            // and
            (0b0110011, 0b111, 0b0000000) => {
                let lhs = self.regs[rs1];
                let rhs = self.regs[rs2];

                self.reg_store(rd, lhs & rhs);
            }

            // andi
            (0b0010011, 0b111, _) => {
                let lhs = self.regs[rs1];
                let rhs = i_imm;

                self.reg_store(rd, lhs & rhs);
            }

            // or
            (0b0110011, 0b110, 0b0000000) => {
                let lhs = self.regs[rs1];
                let rhs = self.regs[rs2];

                self.reg_store(rd, lhs | rhs);
            }

            // ori
            (0b0010011, 0b110, _) => {
                let lhs = self.regs[rs1];
                let rhs = i_imm;

                self.reg_store(rd, lhs | rhs);
            }

            // xor
            (0b0110011, 0b100, 0b0000000) => {
                let lhs = self.regs[rs1];
                let rhs = self.regs[rs2];

                self.reg_store(rd, lhs ^ rhs);
            }

            // xori
            (0b0010011, 0b100, _) => {
                let lhs = self.regs[rs1];
                let rhs = i_imm;

                self.reg_store(rd, lhs ^ rhs);
            }

            // sll
            (0b0110011, 0b001, 0b0000000) => {
                let lhs = self.regs[rs1] as u64;
                let rhs = self.regs[rs2] as u32;

                self.reg_store(rd, lhs.wrapping_shl(rhs) as i64);
            }

            // sllw
            (0b0111011, 0b001, 0b0000000) => {
                let lhs = self.regs[rs1] as u32;
                let rhs = self.regs[rs2] as u32;

                self.reg_store(rd, lhs.wrapping_shl(rhs) as i64);
            }

            // slli
            (0b0010011, 0b001, _) if (i_imm >> 6) == 0x00 => {
                let lhs = self.regs[rs1] as u64;
                let rhs = i_imm as u32;

                self.reg_store(rd, lhs.wrapping_shl(rhs) as i64);
            }

            // srl
            (0b0110011, 0b101, 0b0000000) => {
                let lhs = self.regs[rs1] as u64;
                let rhs = self.regs[rs2] as u32;

                self.reg_store(rd, lhs.wrapping_shr(rhs) as i64);
            }

            // srlw
            (0b0111011, 0b101, 0b0000000) => {
                let lhs = self.regs[rs1] as u32;
                let rhs = self.regs[rs2] as u32;

                self.reg_store(rd, lhs.wrapping_shr(rhs) as i64);
            }

            // srli
            (0b0010011, 0b101, _) if (i_imm >> 6) == 0x00 => {
                let lhs = self.regs[rs1] as u64;
                let rhs = i_imm as u32;

                self.reg_store(rd, lhs.wrapping_shr(rhs) as i64);
            }

            // sra
            (0b0110011, 0b101, 0b0100000) => {
                let lhs = self.regs[rs1];
                let rhs = self.regs[rs2] as u32;

                self.reg_store(rd, lhs.wrapping_shr(rhs));
            }

            // srai
            (0b0010011, 0b101, _) if (i_imm >> 5) == 0x20 => {
                let lhs = self.regs[rs1];
                let rhs = (i_imm as u32) & 0x3f;

                self.reg_store(rd, lhs.wrapping_shr(rhs));
            }

            // slt
            (0b0110011, 0b010, 0b0000000) => {
                let lhs = self.regs[rs1];
                let rhs = self.regs[rs2];

                self.reg_store(rd, (lhs < rhs) as i64);
            }

            // slti
            (0b0010011, 0b010, _) => {
                let lhs = self.regs[rs1];
                let rhs = i_imm;

                self.reg_store(rd, (lhs < rhs) as i64);
            }

            // sltu
            (0b0110011, 0b011, 0b0000000) => {
                let lhs = self.regs[rs1] as u64;
                let rhs = self.regs[rs2] as u64;

                self.reg_store(rd, (lhs < rhs) as i64);
            }

            // sltiu
            (0b0010011, 0b011, _) => {
                let lhs = self.regs[rs1] as u64;
                let rhs = i_imm as u64;

                self.reg_store(rd, (lhs < rhs) as i64);
            }

            // -----------------------------------------------------------------

            // lb
            (0b0000011, 0b000, _) => {
                let addr = (self.regs[rs1] + i_imm) as u64;
                let val = self.mem_load::<1>(mmio, addr)? as i8 as i64;

                self.reg_store(rd, val);
            }

            // lbu
            (0b0000011, 0b100, _) => {
                let addr = (self.regs[rs1] + i_imm) as u64;
                let val = self.mem_load::<1>(mmio, addr)?;

                self.reg_store(rd, val);
            }

            // lh
            (0b0000011, 0b001, _) => {
                let addr = (self.regs[rs1] + i_imm) as u64;
                let val = self.mem_load::<2>(mmio, addr)? as i16 as i64;

                self.reg_store(rd, val);
            }

            // lhu
            (0b0000011, 0b101, _) => {
                let addr = (self.regs[rs1] + i_imm) as u64;
                let val = self.mem_load::<2>(mmio, addr)?;

                self.reg_store(rd, val);
            }

            // lw
            (0b0000011, 0b010, _) => {
                let addr = (self.regs[rs1] + i_imm) as u64;
                let val = self.mem_load::<4>(mmio, addr)? as i32 as i64;

                self.reg_store(rd, val);
            }

            // lwu
            (0b0000011, 0b110, _) => {
                let addr = (self.regs[rs1] + i_imm) as u64;
                let val = self.mem_load::<4>(mmio, addr)?;

                self.reg_store(rd, val);
            }

            // ld
            (0b0000011, 0b011, _) => {
                let addr = (self.regs[rs1] + i_imm) as u64;
                let val = self.mem_load::<8>(mmio, addr)?;

                self.reg_store(rd, val);
            }

            // -----------------------------------------------------------------

            // sb
            (0b0100011, 0b000, _) => {
                self.mem_store::<1>(
                    mmio,
                    self.regs[rs1].wrapping_add(s_imm) as u64,
                    self.regs[rs2],
                )?;
            }

            // sh
            (0b0100011, 0b001, _) => {
                self.mem_store::<2>(
                    mmio,
                    self.regs[rs1].wrapping_add(s_imm) as u64,
                    self.regs[rs2],
                )?;
            }

            // sw
            (0b0100011, 0b010, _) => {
                self.mem_store::<4>(
                    mmio,
                    self.regs[rs1].wrapping_add(s_imm) as u64,
                    self.regs[rs2],
                )?;
            }

            // sd
            (0b0100011, 0b011, _) => {
                self.mem_store::<8>(
                    mmio,
                    self.regs[rs1].wrapping_add(s_imm) as u64,
                    self.regs[rs2],
                )?;
            }

            // -----------------------------------------------------------------

            // TODO implement more atomic instructions

            // amoadd.w
            (0b0101111, 0b010, _) if funct5 == 0b00000 => {
                self.do_atomic::<4>(mmio, rd, rs1, rs2, |lhs, rhs| {
                    (lhs as i32).wrapping_add(rhs as i32) as i64
                })?;
            }

            // amoand.w
            (0b0101111, 0b010, _) if funct5 == 0b01100 => {
                self.do_atomic::<4>(mmio, rd, rs1, rs2, |lhs, rhs| lhs & rhs)?;
            }

            // amoor.w
            (0b0101111, 0b010, _) if funct5 == 0b01000 => {
                self.do_atomic::<4>(mmio, rd, rs1, rs2, |lhs, rhs| lhs | rhs)?;
            }

            // fence
            (0b0001111, 0b000, _) => {
                //
            }

            // -----------------------------------------------------------------

            // beq
            (0b1100011, 0b000, _) => {
                self.do_branch(rs1, rs2, b_imm, |lhs, rhs| lhs == rhs);
            }

            // bne
            (0b1100011, 0b001, _) => {
                self.do_branch(rs1, rs2, b_imm, |lhs, rhs| lhs != rhs);
            }

            // blt
            (0b1100011, 0b100, _) => {
                self.do_branch(rs1, rs2, b_imm, |lhs, rhs| lhs < rhs);
            }

            // bltu
            (0b1100011, 0b110, _) => {
                self.do_branch(rs1, rs2, b_imm, |lhs, rhs| {
                    (lhs as u64) < (rhs as u64)
                });
            }

            // bge
            (0b1100011, 0b101, _) => {
                self.do_branch(rs1, rs2, b_imm, |lhs, rhs| lhs >= rhs);
            }

            // bgeu
            (0b1100011, 0b111, _) => {
                self.do_branch(rs1, rs2, b_imm, |lhs, rhs| {
                    (lhs as u64) >= (rhs as u64)
                });
            }

            // -----------------------------------------------------------------

            // jal
            (0b1101111, _, _) => {
                #[cfg(test)]
                if j_imm == 0 {
                    return Err(anyhow!("infinite loop detected"));
                }

                self.reg_store(rd, self.pc as i64);

                self.pc =
                    self.pc.wrapping_add_signed(j_imm as i64).wrapping_sub(4);
            }

            // jalr
            (0b1100111, 0b000, _) => {
                let rs1_val = self.regs[rs1];

                self.reg_store(rd, self.pc as i64);
                self.pc = rs1_val.wrapping_add(i_imm) as u64;
            }

            // -----------------------------------------------------------------

            // ebreak
            (0b1110011, 0b000, _) if i_imm == 1 => {
                return Ok(false);
            }

            _ => {
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
        // TODO forbid atomic mmio

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
