use crate::{
    cpu::{
        instructions::{Instruction, InstructionEffect, InstructionResult, trap},
        opcode::Opcode,
        operands::Operands,
    },
    exception::Exception,
    system::System,
};

#[macro_export]
macro_rules! decode_special_x {
    ($opcode:expr, $m:ident) => {{
        match $opcode.0 & 0x3F {
            0x00 => $m!(crate::cpu::instructions::special::Sll),
            0x02 => $m!(crate::cpu::instructions::special::Srl),
            0x03 => $m!(crate::cpu::instructions::special::Sra),
            0x04 => $m!(crate::cpu::instructions::special::Sllv),
            0x06 => $m!(crate::cpu::instructions::special::Srlv),
            0x07 => $m!(crate::cpu::instructions::special::Srav),
            0x08 => $m!(crate::cpu::instructions::special::Jr),
            0x09 => $m!(crate::cpu::instructions::special::Jalr),
            0x0C => $m!(crate::cpu::instructions::special::Syscall),
            0x0D => $m!(crate::cpu::instructions::special::Break),
            0x0F => $m!(crate::cpu::instructions::special::Sync),
            0x10 => $m!(crate::cpu::instructions::special::Mfhi),
            0x11 => $m!(crate::cpu::instructions::special::Mthi),
            0x12 => $m!(crate::cpu::instructions::special::Mflo),
            0x13 => $m!(crate::cpu::instructions::special::Mtlo),
            0x14 => $m!(crate::cpu::instructions::special::Dsllv),
            0x16 => $m!(crate::cpu::instructions::special::Dsrlv),
            0x17 => $m!(crate::cpu::instructions::special::Dsrav),
            0x18 => $m!(crate::cpu::instructions::special::Mult),
            0x19 => $m!(crate::cpu::instructions::special::Multu),
            0x1A => $m!(crate::cpu::instructions::special::Div),
            0x1B => $m!(crate::cpu::instructions::special::Divu),
            0x1C => $m!(crate::cpu::instructions::special::Dmult),
            0x1D => $m!(crate::cpu::instructions::special::Dmultu),
            0x1E => $m!(crate::cpu::instructions::special::Ddiv),
            0x1F => $m!(crate::cpu::instructions::special::Ddivu),
            0x20 => $m!(crate::cpu::instructions::special::Add),
            0x21 => $m!(crate::cpu::instructions::special::Addu),
            0x22 => $m!(crate::cpu::instructions::special::Sub),
            0x23 => $m!(crate::cpu::instructions::special::Subu),
            0x24 => $m!(crate::cpu::instructions::special::And),
            0x25 => $m!(crate::cpu::instructions::special::Or),
            0x26 => $m!(crate::cpu::instructions::special::Xor),
            0x27 => $m!(crate::cpu::instructions::special::Nor),
            0x2A => $m!(crate::cpu::instructions::special::Slt),
            0x2B => $m!(crate::cpu::instructions::special::Sltu),
            0x2C => $m!(crate::cpu::instructions::special::Dadd),
            0x2D => $m!(crate::cpu::instructions::special::Daddu),
            0x2E => $m!(crate::cpu::instructions::special::Dsub),
            0x2F => $m!(crate::cpu::instructions::special::Dsubu),
            0x30 => $m!(crate::cpu::instructions::special::Tge),
            0x31 => $m!(crate::cpu::instructions::special::Tgeu),
            0x32 => $m!(crate::cpu::instructions::special::Tlt),
            0x33 => $m!(crate::cpu::instructions::special::Tltu),
            0x34 => $m!(crate::cpu::instructions::special::Teq),
            0x36 => $m!(crate::cpu::instructions::special::Tne),
            0x38 => $m!(crate::cpu::instructions::special::Dsll),
            0x3A => $m!(crate::cpu::instructions::special::Dsrl),
            0x3B => $m!(crate::cpu::instructions::special::Dsra),
            0x3C => $m!(crate::cpu::instructions::special::Dsll32),
            0x3E => $m!(crate::cpu::instructions::special::Dsrl32),
            0x3F => $m!(crate::cpu::instructions::special::Dsra32),
            _ => $m!(crate::cpu::instructions::Reserved),
        }
    }};
}

// -----------
// Arithmetics
// -----------

pub struct Add;

impl Instruction for Add {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        let rs = operands.rsv(s) as i32;
        let rt = operands.rtv(s) as i32;

        match rs.checked_add(rt) {
            Some(result) => {
                s.cpu.regs.gpr[operands.rd()].set(result as u32);
                Ok(None)
            }
            None => Err(Exception::ArithmeticOverflow),
        }
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "ADD {}, {}, {}",
            operands.rdn(),
            operands.rsn(),
            operands.rtn()
        )
    }
}

pub struct Addu;

impl Instruction for Addu {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.gpr[operands.rd()].set(operands.rsv(s).wrapping_add(operands.rtv(s)));

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "ADDU {}, {}, {}",
            operands.rdn(),
            operands.rsn(),
            operands.rtn()
        )
    }
}

pub struct Dadd;

impl Instruction for Dadd {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        let rs = operands.rsv64(s) as i64;
        let rt = operands.rtv64(s) as i64;

        match rs.checked_add(rt) {
            Some(result) => {
                s.cpu.regs.gpr[operands.rd()].set64(result as u64);
                Ok(None)
            }
            None => Err(Exception::ArithmeticOverflow),
        }
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "DADD {}, {}, {}",
            operands.rdn(),
            operands.rsn(),
            operands.rtn()
        )
    }
}

pub struct Daddu;

impl Instruction for Daddu {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.gpr[operands.rd()].set64(operands.rsv64(s).wrapping_add(operands.rtv64(s)));

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "DADDU {}, {}, {}",
            operands.rdn(),
            operands.rsn(),
            operands.rtn()
        )
    }
}

pub struct Sub;

impl Instruction for Sub {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        let rs = operands.rsv(s) as i32;
        let rt = operands.rtv(s) as i32;
        match rs.checked_sub(rt) {
            Some(result) => {
                s.cpu.regs.gpr[operands.rd()].set(result as u32);
                Ok(None)
            }
            None => Err(Exception::ArithmeticOverflow),
        }
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "SUB {}, {}, {}",
            operands.rdn(),
            operands.rsn(),
            operands.rtn()
        )
    }
}

pub struct Subu;

impl Instruction for Subu {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.gpr[operands.rd()].set(operands.rsv(s).wrapping_sub(operands.rtv(s)));

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "SUBU {}, {}, {}",
            operands.rdn(),
            operands.rsn(),
            operands.rtn()
        )
    }
}

pub struct Dsub;

impl Instruction for Dsub {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        let rs = operands.rsv64(s) as i64;
        let rt = operands.rtv64(s) as i64;

        match rs.checked_sub(rt) {
            Some(result) => {
                s.cpu.regs.gpr[operands.rd()].set64(result as u64);
                Ok(None)
            }
            None => Err(Exception::ArithmeticOverflow),
        }
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "DSUB {}, {}, {}",
            operands.rdn(),
            operands.rsn(),
            operands.rtn()
        )
    }
}

pub struct Dsubu;

impl Instruction for Dsubu {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.gpr[operands.rd()].set64(operands.rsv64(s).wrapping_sub(operands.rtv64(s)));

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "DSUBU {}, {}, {}",
            operands.rdn(),
            operands.rsn(),
            operands.rtn()
        )
    }
}

pub struct Mult;

impl Instruction for Mult {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        let result = (operands.rsv(s) as i32 as i64).wrapping_mul(operands.rtv(s) as i32 as i64);

        s.cpu.regs.mult_hi.set((result >> 32) as u32);
        s.cpu.regs.mult_lo.set(result as u32);

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("MULT {}, {}", operands.rsn(), operands.rtn())
    }
}

pub struct Multu;

impl Instruction for Multu {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        let result = (operands.rsv(s) as u64) * (operands.rtv(s) as u64);

        s.cpu.regs.mult_hi.set((result >> 32) as u32);
        s.cpu.regs.mult_lo.set(result as u32);

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("MULTU {}, {}", operands.rsn(), operands.rtn())
    }
}

// TODO div by zero?

pub struct Div;

impl Instruction for Div {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        let rsvs = operands.rsv(s) as i32;
        let rtvs = operands.rtv(s) as i32;

        if rtvs == 0 {
            s.cpu.regs.mult_hi.set(rsvs as u32);
            s.cpu.regs.mult_lo.set(if rsvs >= 0 { u32::MAX } else { 1 });
            // TODO really? matches lemon tests
        } else {
            s.cpu
                .regs
                .mult_hi
                .set((rsvs).overflowing_rem(rtvs).0 as u32);
            s.cpu
                .regs
                .mult_lo
                .set((rsvs).overflowing_div(rtvs).0 as u32);
        }

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("DIV {}, {}", operands.rsn(), operands.rtn())
    }
}

pub struct Divu;

impl Instruction for Divu {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        let rsv = operands.rsv(s);
        let rtv = operands.rtv(s);

        if rtv == 0 {
            s.cpu.regs.mult_hi.set(rsv);
            s.cpu.regs.mult_lo.set(u32::MAX);
        } else {
            s.cpu.regs.mult_hi.set((rsv).overflowing_rem(rtv).0);
            s.cpu.regs.mult_lo.set((rsv).overflowing_div(rtv).0);
        }

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("DIVU {}, {}", operands.rsn(), operands.rtn())
    }
}

pub struct Ddiv;

impl Instruction for Ddiv {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        let rsv = operands.rsv64(s) as i64;
        let rtv = operands.rtv64(s) as i64;

        if rtv == 0 {
            s.cpu.regs.mult_hi.set64(rsv as u64);

            s.cpu
                .regs
                .mult_lo
                .set64(if rsv >= 0 { u64::MAX } else { 1 }); // TODO????
        } else {
            s.cpu
                .regs
                .mult_hi
                .set64((rsv).overflowing_rem(rtv).0 as u64);

            s.cpu
                .regs
                .mult_lo
                .set64((rsv).overflowing_div(rtv).0 as u64);
        }

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("DDIV {}, {}", operands.rsn(), operands.rtn())
    }
}

pub struct Ddivu;

impl Instruction for Ddivu {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        let rs = operands.rsv64(s);
        let rt = operands.rtv64(s);

        if rt == 0 {
            s.cpu.regs.mult_hi.set64(rs);
            s.cpu.regs.mult_lo.set64(u64::MAX);
        } else {
            s.cpu.regs.mult_hi.set64((rs).overflowing_rem(rt).0);
            s.cpu.regs.mult_lo.set64((rs).overflowing_div(rt).0);
        }

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("DDIVU {}, {}", operands.rsn(), operands.rtn())
    }
}

pub struct Dmult;

impl Instruction for Dmult {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        let result = (operands.rsv64(s) as i64 as i128) * (operands.rtv64(s) as i64 as i128);

        s.cpu.regs.mult_hi.set64((result >> 64) as u64);
        s.cpu.regs.mult_lo.set64(result as u64);

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("DMULT {}, {}", operands.rsn(), operands.rtn())
    }
}

pub struct Dmultu;

impl Instruction for Dmultu {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        let result = (operands.rsv64(s) as u128) * (operands.rtv64(s) as u128);

        s.cpu.regs.mult_hi.set64((result >> 64) as u64);
        s.cpu.regs.mult_lo.set64(result as u64);

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("DMULTU {}, {}", operands.rsn(), operands.rtn())
    }
}

// -------
// Boolean
// -------

pub struct And;

impl Instruction for And {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.gpr[operands.rd()].set64(operands.rsv64(s) & operands.rtv64(s));

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "AND {}, {}, {}",
            operands.rdn(),
            operands.rsn(),
            operands.rtn()
        )
    }
}

pub struct Or;

impl Instruction for Or {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.gpr[operands.rd()].set64(operands.rsv64(s) | operands.rtv64(s));

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "OR {}, {}, {}",
            operands.rdn(),
            operands.rsn(),
            operands.rtn()
        )
    }
}

pub struct Nor;

impl Instruction for Nor {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.gpr[operands.rd()].set64(!(operands.rsv64(s) | operands.rtv64(s)));

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "NOR {}, {}, {}",
            operands.rdn(),
            operands.rsn(),
            operands.rtn()
        )
    }
}

pub struct Xor;

impl Instruction for Xor {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.gpr[operands.rd()].set64(operands.rsv64(s) ^ operands.rtv64(s));

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "XOR {}, {}, {}",
            operands.rdn(),
            operands.rsn(),
            operands.rtn()
        )
    }
}

// ------
// Shifts
// ------

pub struct Sra;

impl Instruction for Sra {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        let res = (operands.rtv64(s) >> operands.shift()) as i32 as i64 as u64;
        s.cpu.regs.gpr[operands.rd()].set64(res);

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "SRA {}, {}, {}",
            operands.rdn(),
            operands.rtn(),
            operands.shift()
        )
    }
}

pub struct Srav;

impl Instruction for Srav {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        let res = (operands.rtv64(s) >> (operands.rsv(s) & 0x1F)) as i32 as i64 as u64;
        s.cpu.regs.gpr[operands.rd()].set64(res);

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "SRAV {}, {}, {}",
            operands.rdn(),
            operands.rtn(),
            operands.rsn()
        )
    }
}

pub struct Srl;

impl Instruction for Srl {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.gpr[operands.rd()].set(operands.rtv(s) >> operands.shift());

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "SRL {}, {}, {}",
            operands.rdn(),
            operands.rtn(),
            operands.shift()
        )
    }
}

pub struct Srlv;

impl Instruction for Srlv {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.gpr[operands.rd()].set(operands.rtv(s) >> (operands.rsv(s) & 0x1F));

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "SRLV {}, {}, {}",
            operands.rdn(),
            operands.rtn(),
            operands.rsn()
        )
    }
}

pub struct Dsll;

impl Instruction for Dsll {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        let data = operands.rtv64(s) << operands.shift();

        s.cpu.regs.gpr[operands.rd()].set64(data);

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "DSLL {}, {}, {}",
            operands.rdn(),
            operands.rtn(),
            operands.shift()
        )
    }
}

pub struct Dsll32;

impl Instruction for Dsll32 {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        let data = operands.rtv64(s) << (operands.shift() + 32);

        s.cpu.regs.gpr[operands.rd()].set64(data);

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "DSLL32 {}, {}, {}",
            operands.rdn(),
            operands.rtn(),
            operands.shift()
        )
    }
}

pub struct Dsllv;

impl Instruction for Dsllv {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        let data = operands.rtv64(s) << (operands.rsv(s) & 0x3F);

        s.cpu.regs.gpr[operands.rd()].set64(data);

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "DSLLV {}, {}, {}",
            operands.rdn(),
            operands.rtn(),
            operands.rsn()
        )
    }
}

pub struct Dsra;

impl Instruction for Dsra {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        let data = (operands.rtv64(s) as i64 >> operands.shift()) as u64;

        s.cpu.regs.gpr[operands.rd()].set64(data);

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "DSRA {}, {}, {}",
            operands.rdn(),
            operands.rtn(),
            operands.shift()
        )
    }
}

pub struct Dsra32;

impl Instruction for Dsra32 {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        let data = (operands.rtv64(s) as i64 >> (operands.shift() + 32)) as u64;

        s.cpu.regs.gpr[operands.rd()].set64(data);

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "DSRA32 {}, {}, {}",
            operands.rdn(),
            operands.rtn(),
            operands.shift()
        )
    }
}

pub struct Dsrav;

impl Instruction for Dsrav {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        let data = ((operands.rtv64(s) as i64) >> (operands.rsv(s) & 0x3F)) as u64;

        s.cpu.regs.gpr[operands.rd()].set64(data);

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "DSRAV {}, {}, {}",
            operands.rdn(),
            operands.rtn(),
            operands.rsn()
        )
    }
}

pub struct Dsrl;

impl Instruction for Dsrl {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        let data = operands.rtv64(s) >> operands.shift();

        s.cpu.regs.gpr[operands.rd()].set64(data);

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "DSRL {}, {}, {}",
            operands.rdn(),
            operands.rtn(),
            operands.shift()
        )
    }
}

pub struct Dsrl32;

impl Instruction for Dsrl32 {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        let data = operands.rtv64(s) >> (operands.shift() + 32);
        s.cpu.regs.gpr[operands.rd()].set64(data);

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "DSRL32 {}, {}, {}",
            operands.rdn(),
            operands.rtn(),
            operands.shift()
        )
    }
}

pub struct Dsrlv;

impl Instruction for Dsrlv {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        let data = operands.rtv64(s) >> (operands.rsv(s) & 0x3F);
        s.cpu.regs.gpr[operands.rd()].set64(data);

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "DSRLV {}, {}, {}",
            operands.rdn(),
            operands.rtn(),
            operands.rsn()
        )
    }
}

pub struct Mfhi;

impl Instruction for Mfhi {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.gpr[operands.rd()].set64(s.cpu.regs.mult_hi.get64());

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("MFHI {}", operands.rdn())
    }
}

pub struct Mflo;

impl Instruction for Mflo {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.gpr[operands.rd()].set64(s.cpu.regs.mult_lo.get64());

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("MFLO {}", operands.rdn())
    }
}

pub struct Mthi;

impl Instruction for Mthi {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.mult_hi.set64(operands.rsv64(s));

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("MTHI {}", operands.rsn())
    }
}

pub struct Mtlo;

impl Instruction for Mtlo {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.mult_lo.set64(operands.rsv64(s));

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("MTLO {}", operands.rsn())
    }
}

pub struct Sll;

impl Instruction for Sll {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.gpr[operands.rd()].set(operands.rtv(s) << operands.shift());

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        if operands.rd() == 0 && operands.rt() == 0 {
            "NOP".to_string()
        } else {
            format!(
                "SLL {}, {}, {}",
                operands.rdn(),
                operands.rtn(),
                operands.shift()
            )
        }
    }
}

pub struct Sllv;

impl Instruction for Sllv {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.gpr[operands.rd()].set(operands.rtv(s) << (operands.rsv(s) & 0x1F));

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "SLLV {}, {}, {}",
            operands.rdn(),
            operands.rtn(),
            operands.rsn()
        )
    }
}

pub struct Slt;

impl Instruction for Slt {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.gpr[operands.rd()]
            .set64(((operands.rsv64(s) as i64) < (operands.rtv64(s) as i64)) as u64);

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "SLT {}, {}, {}",
            operands.rdn(),
            operands.rsn(),
            operands.rtn()
        )
    }
}

pub struct Sltu;

impl Instruction for Sltu {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.gpr[operands.rd()].set64((operands.rsv64(s) < operands.rtv64(s)) as u64);

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "SLTU {}, {}, {}",
            operands.rdn(),
            operands.rsn(),
            operands.rtn()
        )
    }
}

// -----
// Traps
// -----

pub struct Teq;

impl Instruction for Teq {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        trap(operands.rsv64(s) == operands.rtv64(s))
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("TEQ {}, {}", operands.rsn(), operands.rtn())
    }
}

pub struct Tge;

impl Instruction for Tge {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        trap((operands.rsv64(s) as i64) >= (operands.rtv64(s) as i64))
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("TGE {}, {}", operands.rsn(), operands.rtn())
    }
}

pub struct Tgeu;

impl Instruction for Tgeu {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        trap(operands.rsv64(s) >= operands.rtv64(s))
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("TGEU {}, {}", operands.rsn(), operands.rtn())
    }
}

pub struct Tlt;

impl Instruction for Tlt {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        trap((operands.rsv64(s) as i64) < (operands.rtv64(s) as i64))
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("TLT {}, {}", operands.rsn(), operands.rtn())
    }
}

pub struct Tltu;

impl Instruction for Tltu {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        if operands.rsv64(s) < operands.rtv64(s) {
            Err(Exception::Trap)
        } else {
            Ok(None)
        }
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("TLTU {}, {}", operands.rsn(), operands.rtn())
    }
}

pub struct Tne;

impl Instruction for Tne {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        // TODO trap helper!
        if operands.rsv64(s) != operands.rtv64(s) {
            Err(Exception::Trap)
        } else {
            Ok(None)
        }
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("TNE {}, {}", operands.rsn(), operands.rtn())
    }
}

// -----
// Jumps
// -----

pub struct Jalr;

impl Instruction for Jalr {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        let target = operands.rsv(s);

        s.cpu.regs.gpr[operands.rd()].set(s.cpu.regs.pc.wrapping_add(8));

        Ok(Some(InstructionEffect::DelayedBranching(Some(target))))
    }

    fn disassemble(s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!(
            "JALR {}, {}={:#06X}",
            operands.rdn(),
            operands.rsn(),
            operands.rsv(s)
        )
    }
}

pub struct Jr;

impl Instruction for Jr {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        Ok(Some(InstructionEffect::DelayedBranching(Some(
            operands.rsv(s),
        ))))
    }

    fn disassemble(s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("JR {}={:#06X}", operands.rsn(), operands.rsv(s))
    }
}

// -----
// Misc.
// -----

pub struct Break;

impl Instruction for Break {
    fn execute(_s: &mut System, _opcode: Opcode, _operands: Operands) -> InstructionResult {
        Err(Exception::Breakpoint)
    }

    fn disassemble(_s: &System, _opcode: Opcode, _operands: Operands) -> String {
        "BREAK".to_string()
    }
}

pub struct Syscall;

impl Instruction for Syscall {
    fn execute(_s: &mut System, _opcode: Opcode, _operands: Operands) -> InstructionResult {
        Err(Exception::Syscall)
    }

    fn disassemble(_s: &System, _opcode: Opcode, _operands: Operands) -> String {
        "SYSCALL".to_string()
    }
}

pub struct Sync;

impl Instruction for Sync {
    fn execute(_s: &mut System, _opcode: Opcode, _operands: Operands) -> InstructionResult {
        // TODO?

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, _operands: Operands) -> String {
        "SYNC".to_string()
    }
}
