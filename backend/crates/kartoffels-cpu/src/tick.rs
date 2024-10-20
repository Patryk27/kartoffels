use super::{Cpu, Mmio};

macro_rules! arg {
    ($word:expr => $($val:ident),*) => {
        $(
            let $val = arg!(@ $word, $val);
        )*
    };

    (@ $word:expr, rd) => {
        (($word >> 7) & 0x1f) as usize
    };

    (@ $word:expr, rs1) => {
        (($word >> 15) & 0x1f) as usize
    };

    (@ $word:expr, rs2) => {
        (($word >> 20) & 0x1f) as usize
    };

    (@ $word:expr, i_imm) => {
        ($word as i32 as i64) >> 20
    };

    (@ $word:expr, u_imm) => {
        ($word as i32 as i64) >> 12
    };

    (@ $word:expr, s_imm) => {
        (($word & 0xfe000000) as i32 as i64 >> 20)
            | ((($word >> 7) & 0x1f) as i64)
    };

    (@ $word:expr, b_imm) => {
        ((($word & 0x80000000) as i32 >> 19) as u32
            | (($word & 0x80) << 4)
            | (($word >> 20) & 0x7e0)
            | (($word >> 7) & 0x1e)) as i32
    };

    (@ $word:expr, j_imm) => {
        ((($word & 0x80000000) as i32 >> 11) as u32
            | ($word & 0xff000)
            | (($word >> 9) & 0x800)
            | (($word >> 20) & 0x7fe)) as i32
    };
}

impl Cpu {
    pub(super) fn do_tick(
        &mut self,
        mmio: &mut dyn Mmio,
    ) -> Result<(), Box<str>> {
        let word = self.mem_load::<4>(mmio, self.pc)? as u32;

        let op = word & 0x7f;
        let funct3 = (word >> 12) & 0x7;
        let funct5 = word >> 27;
        let funct7 = word >> 25;

        self.pc += 4;

        macro_rules! op {
            ($name:ident ( $self:ident, $($arg:ident),* ) $body:tt) => {{
                #[inline(never)]
                fn $name($self: &mut Cpu, word: u32) {
                    arg!(word => $($arg),*);
                    $body
                }

                $name(self, word);
            }};

            ($name:ident ( $self:ident, $($arg:ident),* ) try $body:tt) => {{
                #[inline(never)]
                fn $name($self: &mut Cpu, word: u32) -> Result<(), Box<str>> {
                    arg!(word => $($arg),*);
                    $body;

                    Ok(())
                }

                $name(self, word)?;
            }};

            ($name:ident ( $self:ident+$mmio:ident, $($arg:ident),* ) $body:tt) => {{
                #[inline(never)]
                fn $name($self: &mut Cpu, $mmio: &mut dyn Mmio, word: u32) {
                    arg!(word => $($arg),*);
                    $body
                }

                $name(self, mmio, word);
            }};

            ($name:ident ( $self:ident+$mmio:ident, $($arg:ident),* ) try $body:tt) => {{
                #[inline(never)]
                fn $name(
                    $self: &mut Cpu,
                    $mmio: &mut dyn Mmio,
                    word: u32,
                ) -> Result<(), Box<str>> {
                    arg!(word => $($arg),*);
                    $body;

                    Ok(())
                }

                $name(self, mmio, word)?;
            }};
        }

        macro_rules! err_unknown {
            () => {
                format!("unknown instruction: 0x{word:08x}").into()
            };
        }

        match (op, funct3, funct7) {
            (0b0110111, _, _) => op!(
                lui(this, rd, u_imm) {
                    this.reg_store(rd, u_imm << 12);
                }
            ),

            (0b0010111, _, _) => op!(
                auipc(this, rd, u_imm) {
                    this.reg_store(rd, (this.pc as i64) - 4 + (u_imm << 12));
                }
            ),

            (0b0110011, 0b000, 0b0000000) => op!(
                add(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1];
                    let rhs = this.regs[rs2];

                    this.reg_store(rd, lhs.wrapping_add(rhs));
                }
            ),

            (0b0111011, 0b000, 0b0000000) => op!(
                addw(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1];
                    let rhs = this.regs[rs2];

                    this.reg_store(rd, lhs.wrapping_add(rhs) as i32 as i64);
                }
            ),

            (0b0010011, 0b000, _) => op!(
                addi(this, rd, rs1, i_imm) {
                    let lhs = this.regs[rs1];
                    let rhs = i_imm;

                    this.reg_store(rd, lhs.wrapping_add(rhs));
                }
            ),

            (0b0011011, 0b000, _) => op!(
                addiw(this, rd, rs1, i_imm) {
                    let lhs = this.regs[rs1];
                    let rhs = i_imm;

                    this.reg_store(rd, lhs.wrapping_add(rhs) as i32 as i64);
                }
            ),

            (0b0110011, 0b000, 0b0100000) => op!(
                sub(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1];
                    let rhs = this.regs[rs2];

                    this.reg_store(rd, lhs.wrapping_sub(rhs));
                }
            ),

            (0b0111011, 0b000, 0b0100000) => op!(
                subw(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1];
                    let rhs = this.regs[rs2];

                    this.reg_store(rd, lhs.wrapping_sub(rhs) as i32 as i64);
                }
            ),

            (0b0110011, 0b000, 0b0000001) => op!(
                mul(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1];
                    let rhs = this.regs[rs2];

                    this.reg_store(rd, lhs.wrapping_mul(rhs));
                }
            ),

            (0b0110011, 0b001, 0b0000001) => op!(
                mulh(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1] as i128;
                    let rhs = this.regs[rs2] as i128;

                    this.reg_store(rd, (lhs.wrapping_mul(rhs) >> 64) as i64);
                }
            ),

            (0b0110011, 0b010, 0b0000001) => op!(
                mulhsu(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1] as i128 as u128;
                    let rhs = this.regs[rs2] as u64 as u128;

                    this.reg_store(rd, (lhs.wrapping_mul(rhs) >> 64) as u64 as i64);
                }
            ),

            (0b0110011, 0b011, 0b0000001) => op!(
                mulhu(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1] as u64 as u128;
                    let rhs = this.regs[rs2] as u64 as u128;

                    this.reg_store(rd, (lhs.wrapping_mul(rhs) >> 64) as i64);
                }
            ),

            (0b0111011, 0b000, 0b0000001) => op!(
                mulw(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1];
                    let rhs = this.regs[rs2];

                    this.reg_store(rd, lhs.wrapping_mul(rhs) as i32 as i64);
                }
            ),

            (0b0110011, 0b100, 0b0000001) => op!(
                div(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1];
                    let rhs = this.regs[rs2];

                    this.reg_store(rd, lhs.checked_div(rhs).unwrap_or(-1));
                }
            ),

            (0b0111011, 0b100, 0b0000001) => op!(
                divw(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1] as i32;
                    let rhs = this.regs[rs2] as i32;

                    this.reg_store(rd, lhs.checked_div(rhs).unwrap_or(-1) as i64);
                }
            ),

            (0b0110011, 0b101, 0b0000001) => op!(
                divu(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1] as u64;
                    let rhs = this.regs[rs2] as u64;

                    this.reg_store(
                        rd,
                        lhs.checked_div(rhs).unwrap_or(-1i64 as u64) as i64,
                    );
                }
            ),

            (0b0111011, 0b101, 0b0000001) => op!(
                divuv(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1] as u32;
                    let rhs = this.regs[rs2] as u32;

                    this.reg_store(
                        rd,
                        lhs.checked_div(rhs)
                            .map(|val| val as i32 as i64)
                            .unwrap_or(-1),
                    );
                }
            ),

            (0b0110011, 0b110, 0b0000001) => op!(
                rem(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1];
                    let rhs = this.regs[rs2];

                    this.reg_store(rd, lhs.checked_rem(rhs).unwrap_or(-1));
                }
            ),

            (0b0111011, 0b110, 0b0000001) => op!(
                remw(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1] as i32;
                    let rhs = this.regs[rs2] as i32;

                    this.reg_store(rd, lhs.checked_rem(rhs).unwrap_or(-1) as i64);
                }
            ),

            (0b0110011, 0b111, 0b0000001) => op!(
                remu(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1] as u64;
                    let rhs = this.regs[rs2] as u64;

                    this.reg_store(
                        rd,
                        lhs.checked_rem(rhs).unwrap_or(-1i64 as u64) as i64,
                    );
                }
            ),

            (0b0111011, 0b111, 0b0000001) => op!(
                remuw(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1] as u32;
                    let rhs = this.regs[rs2] as u32;

                    this.reg_store(
                        rd,
                        lhs.checked_rem(rhs)
                            .map(|val| val as i32 as i64)
                            .unwrap_or(-1),
                    );
                }
            ),

            (0b0110011, 0b111, 0b0000000) => op!(
                and(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1];
                    let rhs = this.regs[rs2];

                    this.reg_store(rd, lhs & rhs);
                }
            ),

            (0b0010011, 0b111, _) => op!(
                andi(this, rd, rs1, i_imm) {
                    let lhs = this.regs[rs1];
                    let rhs = i_imm;

                    this.reg_store(rd, lhs & rhs);
                }
            ),

            (0b0110011, 0b110, 0b0000000) => op!(
                or(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1];
                    let rhs = this.regs[rs2];

                    this.reg_store(rd, lhs | rhs);
                }
            ),

            (0b0010011, 0b110, _) => op!(
                ori(this, rd, rs1, i_imm) {
                    let lhs = this.regs[rs1];
                    let rhs = i_imm;

                    this.reg_store(rd, lhs | rhs);
                }
            ),

            (0b0110011, 0b100, 0b0000000) => op!(
                xor(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1];
                    let rhs = this.regs[rs2];

                    this.reg_store(rd, lhs ^ rhs);
                }
            ),

            (0b0010011, 0b100, _) => op!(
                xori(this, rd, rs1, i_imm) {
                    let lhs = this.regs[rs1];
                    let rhs = i_imm;

                    this.reg_store(rd, lhs ^ rhs);
                }
            ),

            (0b0110011, 0b001, 0b0000000) => op!(
                sll(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1] as u64;
                    let rhs = this.regs[rs2] as u32;

                    this.reg_store(rd, lhs.wrapping_shl(rhs) as i64);
                }
            ),

            (0b0111011, 0b001, 0b0000000) => op!(
                sllw(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1] as u32;
                    let rhs = this.regs[rs2] as u32;

                    this.reg_store(rd, lhs.wrapping_shl(rhs) as i32 as i64);
                }
            ),

            (0b0010011, 0b001, _) => {
                arg!(word => i_imm);

                match i_imm >> 6 {
                    0x00 => op!(
                        slli(this, rd, rs1, i_imm) {
                            let lhs = this.regs[rs1] as u64;
                            let rhs = i_imm as u32;

                            this.reg_store(rd, lhs.wrapping_shl(rhs) as i64);
                        }
                    ),

                    _ => {
                        return Err(err_unknown!());
                    }
                }
            }

            (0b0011011, 0b001, _) => {
                arg!(word => i_imm);

                match i_imm >> 6 {
                    0x00 => op!(
                        slliw(this, rd, rs1, i_imm) {
                            let lhs = this.regs[rs1] as u64;
                            let rhs = i_imm as u32;

                            this.reg_store(rd, lhs.wrapping_shl(rhs) as i32 as i64);
                        }
                    ),

                    _ => {
                        return Err(err_unknown!());
                    }
                }
            }

            (0b0110011, 0b101, 0b0000000) => op!(
                srl(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1] as u64;
                    let rhs = this.regs[rs2] as u32;

                    this.reg_store(rd, lhs.wrapping_shr(rhs) as i64);
                }
            ),

            (0b0111011, 0b101, 0b0000000) => op!(
                srlw(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1] as u32;
                    let rhs = this.regs[rs2] as u32;

                    this.reg_store(rd, lhs.wrapping_shr(rhs) as i32 as i64);
                }
            ),

            (0b0010011, 0b101, _) => {
                arg!(word => i_imm);

                match i_imm >> 6 {
                    0x00 => op!(
                        srli(this, rd, rs1, i_imm) {
                            let lhs = this.regs[rs1] as u64;
                            let rhs = i_imm as u32;

                            this.reg_store(rd, lhs.wrapping_shr(rhs) as i64);
                        }
                    ),

                    0x10 => op!(
                        srai(this, rd, rs1, i_imm) {
                            let lhs = this.regs[rs1];
                            let rhs = (i_imm as u32) & 0x3f;

                            this.reg_store(rd, lhs.wrapping_shr(rhs));
                        }
                    ),

                    _ => {
                        return Err(err_unknown!());
                    }
                }
            }

            (0b0011011, 0b101, _) => {
                arg!(word => i_imm);

                match i_imm >> 6 {
                    0x00 => op!(
                        srliw(this, rd, rs1, i_imm) {
                            let lhs = this.regs[rs1] as u32;
                            let rhs = i_imm as u32;

                            this.reg_store(rd, lhs.wrapping_shr(rhs) as i32 as i64);
                        }
                    ),

                    0x10 => op!(
                        sraiw(this, rd, rs1, i_imm) {
                            let lhs = this.regs[rs1] as i32;
                            let rhs = (i_imm as u32) & 0x3f;

                            this.reg_store(rd, lhs.wrapping_shr(rhs) as i64);
                        }
                    ),

                    _ => {
                        return Err(err_unknown!());
                    }
                }
            }

            (0b0110011, 0b101, 0b0100000) => op!(
                sra(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1];
                    let rhs = this.regs[rs2] as u32;

                    this.reg_store(rd, lhs.wrapping_shr(rhs));
                }
            ),

            (0b0111011, 0b101, 0b0100000) => op!(
                sraw(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1] as i32;
                    let rhs = this.regs[rs2] as u32;

                    this.reg_store(rd, lhs.wrapping_shr(rhs) as i64);
                }
            ),

            (0b0110011, 0b010, 0b0000000) => op!(
                slt(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1];
                    let rhs = this.regs[rs2];

                    this.reg_store(rd, (lhs < rhs) as i64);
                }
            ),

            (0b0010011, 0b010, _) => op!(
                slti(this, rd, rs1, i_imm) {
                    let lhs = this.regs[rs1];
                    let rhs = i_imm;

                    this.reg_store(rd, (lhs < rhs) as i64);
                }
            ),

            (0b0110011, 0b011, 0b0000000) => op!(
                sltu(this, rd, rs1, rs2) {
                    let lhs = this.regs[rs1] as u64;
                    let rhs = this.regs[rs2] as u64;

                    this.reg_store(rd, (lhs < rhs) as i64);
                }
            ),

            (0b0010011, 0b011, _) => op!(
                sltiu(this, rd, rs1, i_imm) {
                    let lhs = this.regs[rs1] as u64;
                    let rhs = i_imm as u64;

                    this.reg_store(rd, (lhs < rhs) as i64);
                }
            ),

            (0b0000011, 0b000, _) => op!(
                lb(this+mmio, rd, rs1, i_imm) try {
                    let addr = (this.regs[rs1] + i_imm) as u64;
                    let val = this.mem_load::<1>(mmio, addr)? as i8 as i64;

                    this.reg_store(rd, val);
                }
            ),

            (0b0000011, 0b100, _) => op!(
                lbu(this+mmio, rd, rs1, i_imm) try {
                    let addr = (this.regs[rs1] + i_imm) as u64;
                    let val = this.mem_load::<1>(mmio, addr)?;

                    this.reg_store(rd, val);
                }
            ),

            (0b0000011, 0b001, _) => op!(
                lh(this+mmio, rd, rs1, i_imm) try {
                    let addr = (this.regs[rs1] + i_imm) as u64;
                    let val = this.mem_load::<2>(mmio, addr)? as i16 as i64;

                    this.reg_store(rd, val);
                }
            ),

            (0b0000011, 0b101, _) => op!(
                lhu(this+mmio, rd, rs1, i_imm) try {
                    let addr = (this.regs[rs1] + i_imm) as u64;
                    let val = this.mem_load::<2>(mmio, addr)?;

                    this.reg_store(rd, val);
                }
            ),

            (0b0000011, 0b010, _) => op!(
                lw(this+mmio, rd, rs1, i_imm) try {
                    let addr = (this.regs[rs1] + i_imm) as u64;
                    let val = this.mem_load::<4>(mmio, addr)? as i32 as i64;

                    this.reg_store(rd, val);
                }
            ),

            (0b0000011, 0b110, _) => op!(
                lwu(this+mmio, rd, rs1, i_imm) try {
                    let addr = (this.regs[rs1] + i_imm) as u64;
                    let val = this.mem_load::<4>(mmio, addr)?;

                    this.reg_store(rd, val);
                }
            ),

            (0b0000011, 0b011, _) => op!(
                ld(this+mmio, rd, rs1, i_imm) try {
                    let addr = (this.regs[rs1] + i_imm) as u64;
                    let val = this.mem_load::<8>(mmio, addr)?;

                    this.reg_store(rd, val);
                }
            ),

            (0b0100011, 0b000, _) => op!(
                sb(this+mmio, rs1, rs2, s_imm) try {
                    this.mem_store::<1>(
                        mmio,
                        this.regs[rs1].wrapping_add(s_imm) as u64,
                        this.regs[rs2],
                    )?;
                }
            ),

            (0b0100011, 0b001, _) => op!(
                sh(this+mmio, rs1, rs2, s_imm) try {
                    this.mem_store::<2>(
                        mmio,
                        this.regs[rs1].wrapping_add(s_imm) as u64,
                        this.regs[rs2],
                    )?;
                }
            ),

            (0b0100011, 0b010, _) => op!(
                sw(this+mmio, rs1, rs2, s_imm) try {
                    this.mem_store::<4>(
                        mmio,
                        this.regs[rs1].wrapping_add(s_imm) as u64,
                        this.regs[rs2],
                    )?;
                }
            ),

            (0b0100011, 0b011, _) => op!(
                sd(this+mmio, rs1, rs2, s_imm) try {
                    this.mem_store::<8>(
                        mmio,
                        this.regs[rs1].wrapping_add(s_imm) as u64,
                        this.regs[rs2],
                    )?;
                }
            ),

            (0b0101111, 0b010, _) if funct5 == 0b00000 => op!(
                amoaddw(this+mmio, rd, rs1, rs2) try {
                    this.do_atomic::<4>(mmio, rd, rs1, rs2, |lhs, rhs| {
                        (lhs as i32).wrapping_add(rhs as i32) as i64
                    })?;
                }
            ),

            (0b0101111, 0b010, _) if funct5 == 0b01100 => op!(
                amoandw(this+mmio, rd, rs1, rs2) try {
                    this.do_atomic::<4>(mmio, rd, rs1, rs2, |lhs, rhs| {
                        lhs & rhs
                    })?;
                }
            ),

            (0b0101111, 0b010, _) if funct5 == 0b01000 => op!(
                amoorw(this+mmio, rd, rs1, rs2) try {
                    this.do_atomic::<4>(mmio, rd, rs1, rs2, |lhs, rhs| {
                        lhs | rhs
                    })?;
                }
            ),

            (0b0001111, 0b000, _) => {
                // fence
            }

            (0b1100011, 0b000, _) => op!(
                beq(this, rs1, rs2, b_imm) {
                    this.do_branch(rs1, rs2, b_imm, |lhs, rhs| lhs == rhs);
                }
            ),

            (0b1100011, 0b001, _) => op!(
                bne(this, rs1, rs2, b_imm) {
                    this.do_branch(rs1, rs2, b_imm, |lhs, rhs| lhs != rhs);
                }
            ),

            (0b1100011, 0b100, _) => op!(
                blt(this, rs1, rs2, b_imm) {
                    this.do_branch(rs1, rs2, b_imm, |lhs, rhs| lhs < rhs);
                }
            ),

            (0b1100011, 0b110, _) => op!(
                bltu(this, rs1, rs2, b_imm) {
                    this.do_branch(rs1, rs2, b_imm, |lhs, rhs| {
                        (lhs as u64) < (rhs as u64)
                    });
                }
            ),

            (0b1100011, 0b101, _) => op!(
                bge(this, rs1, rs2, b_imm) {
                    this.do_branch(rs1, rs2, b_imm, |lhs, rhs| lhs >= rhs);
                }
            ),

            (0b1100011, 0b111, _) => op!(
                bgeu(this, rs1, rs2, b_imm) {
                    this.do_branch(rs1, rs2, b_imm, |lhs, rhs| {
                        (lhs as u64) >= (rhs as u64)
                    });
                }
            ),

            (0b1101111, _, _) => op!(
                jal(this, rd, j_imm) try {
                    #[cfg(test)]
                    if j_imm == 0 {
                        return Err("infinite loop detected".into());
                    }

                    this.reg_store(rd, this.pc as i64);

                    this.pc =
                        this.pc.wrapping_add_signed(j_imm as i64).wrapping_sub(4);
                }
            ),

            (0b1100111, 0b000, _) => op!(
                jalr(this, rd, rs1, i_imm) {
                    let rs1_val = this.regs[rs1];

                    this.reg_store(rd, this.pc as i64);
                    this.pc = rs1_val.wrapping_add(i_imm) as u64;
                }
            ),

            (0b1110011, 0b000, _) => {
                arg!(word => i_imm);

                match i_imm {
                    // ebreak
                    0x01 => {
                        return Err("got `ebreak`".into());
                    }

                    _ => {
                        return Err(err_unknown!());
                    }
                }
            }

            _ => {
                return Err(err_unknown!());
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
        mmio: &mut dyn Mmio,
        rd: usize,
        rs1: usize,
        rs2: usize,
        op: fn(i64, i64) -> i64,
    ) -> Result<(), Box<str>> {
        let addr = Self::mem_translate(self.regs[rs1] as u64, SIZE)?;

        if addr >= Self::MMIO_BASE {
            return Err(Self::mem_fault(
                "unsupported atomic mmio operation",
                addr,
                SIZE,
            ));
        }

        let old_val = self.mem_load::<SIZE>(mmio, addr as u64)?;
        let new_val = op(old_val, self.regs[rs2]);

        self.mem_store::<SIZE>(mmio, addr as u64, new_val)?;
        self.reg_store(rd, old_val);

        Ok(())
    }

    fn reg_store(&mut self, id: usize, val: i64) {
        if id != 0 {
            self.regs[id] = val;
        }
    }
}
