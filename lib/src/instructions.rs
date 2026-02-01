use crate::cpu::CPU;
use crate::registers::Registers;

/// Result of a branch/jump: target PC to use after the delay slot.
#[derive(Clone, Copy, Debug)]
pub struct DelayedBranching(pub u64);

/// Instruction trait.
pub trait Instruction {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching>;
    fn disassemble(&self, cpu: &CPU, instruction: u32) -> String;
}

/// Returns the instruction for the given opcode
pub fn decode(opcode: u32) -> &'static dyn Instruction {
    let op = opcode >> 26;

    match op {
        0x00 => match opcode & 0x3F {
            0x00 => &SLL_,
            0x08 => &JR_,
            0x10 => &MFHI_,
            0x12 => &MFLO_,
            0x19 => &MULTU_,
            0x21 => &ADDU_,
            0x24 => &AND_,
            0x25 => &OR_,
            0x26 => &XOR_,
            0x2B => &SLTU_,
            0x20 => &ADD_,
            _ => &UNKNOWN_,
        },
        0x05 => &BNE_,
        0x08 => &ADDI_,
        0x09 => &ADDIU_,
        0x0C => &ANDI_,
        0x0D => &ORI_,
        0x0F => &LUI_,
        0x15 => &BNEL_,
        0x23 => &LW_,
        0x2B => &SW_,
        0x10 => &COP0_,
        0x2F => &CACHE_,
        _ => &UNKNOWN_,
    }
}

/// Macro to define an instruction struct with a static instance (with a _ suffix).
macro_rules! instruction_struct {
    ($NAME:ident) => {
        paste::paste! {
            struct $NAME;
            static [< $NAME _ >]: $NAME = $NAME;
        }
    };
}

instruction_struct!(UNKNOWN);

impl Instruction for UNKNOWN {
    fn execute(&self, _cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        panic!("Unknown opcode: {:08X}", opcode)
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        format!("<UNKNOWN {:08X}>", opcode)
    }
}

instruction_struct!(ADD);

impl Instruction for ADD {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let rd = rd(opcode);
        let rt = rt(opcode);
        let rs = rs(opcode);

        cpu.regs.gpr[rd] = cpu.regs.gpr[rs].wrapping_add(cpu.regs.gpr[rt]);

        None

        // TODO overflow rules
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        let rd = rd(opcode);
        let rt = rt(opcode);
        let rs = rs(opcode);

        format!("ADD {}, {}, {}", reg(rd), reg(rs), reg(rt))
    }
}

instruction_struct!(ADDI);

impl Instruction for ADDI {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let imm = imm16(opcode) as i16 as u64;
        let rt = rt(opcode);
        let rs = rs(opcode);

        cpu.regs.gpr[rt] = cpu.regs.gpr[rs].wrapping_add(imm);

        None

        // TODO overflow rules
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        let imm = imm16(opcode) as i16;
        let rt = rt(opcode);
        let rs = rs(opcode);

        format!("ADDI {}, {}, {:#06X}", rt, rs, imm)
    }
}

instruction_struct!(ADDIU);

impl Instruction for ADDIU {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let imm = imm16_sext_u64(opcode);
        let rt = rt(opcode);
        let rs = rs(opcode);

        cpu.regs.gpr[rt] = cpu.regs.gpr[rs].wrapping_add(imm);

        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        format!(
            "ADDIU {}, {}, {:#X}",
            reg(rt(opcode)),
            reg(rs(opcode)),
            imm16(opcode) as i16
        )
    }
}

instruction_struct!(ADDU);

impl Instruction for ADDU {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let rd = rd(opcode);
        let rt = rt(opcode);
        let rs = rs(opcode);

        cpu.regs.gpr[rd] = cpu.regs.gpr[rs].wrapping_add(cpu.regs.gpr[rt]);

        None

        // TODO no overflow exception
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        let rd = rd(opcode);
        let rt = rt(opcode);
        let rs = rs(opcode);

        format!("ADDU {}, {}, {}", reg(rd), reg(rs), reg(rt))
    }
}

instruction_struct!(AND);

impl Instruction for AND {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[rd(opcode)] = cpu.regs.gpr[rs(opcode)] & cpu.regs.gpr[rt(opcode)];

        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        format!(
            "AND {}, {}, {}",
            reg(rd(opcode)),
            reg(rs(opcode)),
            reg(rt(opcode))
        )
    }
}

instruction_struct!(ANDI);

impl Instruction for ANDI {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let imm = imm16(opcode) as u64;
        let rt = rt(opcode);
        let rs = rs(opcode);

        cpu.regs.gpr[rt] = cpu.regs.gpr[rs] & imm;

        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        let imm = imm16(opcode) as u64;
        let rt = rt(opcode);
        let rs = rs(opcode);

        format!("ANDI {}, {}, {:#06X}", reg(rt), reg(rs), imm)
    }
}

instruction_struct!(BNE);

impl Instruction for BNE {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let rs = rs(opcode);
        let rt = rt(opcode);

        if cpu.regs.gpr[rs] != cpu.regs.gpr[rt] {
            let offset = (imm16(opcode) as i16 as i32 as i64) << 2; // TODO less casts??

            let future_pc = cpu.regs.pc.wrapping_add(4).wrapping_add(offset as u64);

            Some(DelayedBranching(future_pc))
        } else {
            None
        }
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        let offset = (imm16(opcode) as i16 as i32) << 2;

        format!(
            "BNE {}, {}, {:#X}",
            reg(rs(opcode)),
            reg(rt(opcode)),
            offset
        )
    }
}

instruction_struct!(BNEL);

impl Instruction for BNEL {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let rs = rs(opcode);
        let rt = rt(opcode);

        if cpu.regs.gpr[rs] != cpu.regs.gpr[rt] {
            let offset = (imm16(opcode) as i16 as i32 as i64) << 2; // TODO less casts??

            let future_pc = cpu.regs.pc.wrapping_add(4).wrapping_add(offset as u64);

            Some(DelayedBranching(future_pc))
        } else {
            // Skip the delay slot
            cpu.regs.pc = cpu.regs.pc.wrapping_add(4);

            None
        }
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        let offset = (imm16(opcode) as i16 as u32) << 2;

        format!(
            "BNEL {}, {}, {:#X}",
            reg(rs(opcode)),
            reg(rt(opcode)),
            offset
        )
    }
}

instruction_struct!(CACHE);

impl Instruction for CACHE {
    fn execute(&self, _cpu: &mut CPU, _opcode: u32) -> Option<DelayedBranching> {
        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        let op = (opcode >> 16) & 0x1F;
        let base = rs(opcode);

        format!("CACHE {}, {}({})", op, imm16(opcode) as i16, reg(base))
    }
}

instruction_struct!(COP0);

impl Instruction for COP0 {
    fn execute(&self, _cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let sub = (opcode >> 21) & 0x1F;
        match sub {
            4 => {} // MTC0 todo
            _ => panic!("Unknown COP0 sub: {:02x}", sub),
        }
        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        let sub = (opcode >> 21) & 0x1F;

        // TODO

        format!("<COP0 {:02x}>", sub)
    }
}

instruction_struct!(JR);

impl Instruction for JR {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let rs = rs(opcode);
        let addr = cpu.regs.gpr[rs] as u32 as u64;

        Some(DelayedBranching(addr))
    }

    fn disassemble(&self, cpu: &CPU, opcode: u32) -> String {
        let rs = rs(opcode);
        let addr = cpu.regs.gpr[rs] as u32;

        format!("JR {}={:#06X}", reg(rs), addr)
    }
}

instruction_struct!(LUI);

impl Instruction for LUI {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let imm = (opcode & 0xFFFF) as u32; // TODO imm()
        let rt = rt(opcode);

        cpu.regs.gpr[rt] = (imm << 16) as i32 as u64;

        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        let imm = (opcode & 0xFFFF) as u32; // TODO imm()

        format!("LUI {}, {:#X}", reg(rt(opcode)), imm)
    }
}

instruction_struct!(LW);

impl Instruction for LW {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let offset = imm16(opcode) as i16 as i32 as u32; // TODO less casts??
        let rt = rt(opcode);
        let base = rs(opcode);

        let addr = cpu.regs.gpr[base] as u32 + offset;

        cpu.regs.gpr[rt] = cpu.read(addr) as i32 as u64;

        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        let offset = imm16(opcode) as i16;

        format!(
            "LW {}, {:#06X}({})",
            reg(rt(opcode)),
            offset,
            reg(rs(opcode))
        )
    }
}

instruction_struct!(MFHI);

impl Instruction for MFHI {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let rd = rd(opcode);

        cpu.regs.gpr[rd] = cpu.regs.mult_hi;

        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        format!("MFHI {}", reg(rt(opcode)),)
    }
}

instruction_struct!(MFLO);

impl Instruction for MFLO {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let rd = rd(opcode);

        cpu.regs.gpr[rd] = cpu.regs.mult_lo;

        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        format!("MFLO {}", reg(rt(opcode)),)
    }
}

instruction_struct!(MULTU);

impl Instruction for MULTU {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let rt = rt(opcode);
        let rs = rs(opcode);

        let result = (cpu.regs.gpr[rs] as u32 as u64) * (cpu.regs.gpr[rt] as u32 as u64);

        cpu.regs.mult_hi = result >> 32;
        cpu.regs.mult_lo = result & 0xFFFFFFFF;

        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        let rt = rt(opcode);
        let rs = rs(opcode);

        format!("MULTU {}, {}", reg(rs), reg(rt))
    }
}

instruction_struct!(OR);

impl Instruction for OR {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[rd(opcode)] = cpu.regs.gpr[rs(opcode)] | cpu.regs.gpr[rt(opcode)];

        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        format!(
            "OR {}, {}, {}",
            reg(rd(opcode)),
            reg(rs(opcode)),
            reg(rt(opcode))
        )
    }
}

instruction_struct!(ORI);

impl Instruction for ORI {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let imm = imm16(opcode) as u64;
        let rt = rt(opcode);
        let rs = rs(opcode);

        cpu.regs.gpr[rt] = cpu.regs.gpr[rs] | imm;

        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        let imm = imm16(opcode) as u64;
        let rt = rt(opcode);
        let rs = rs(opcode);

        format!("ORI {}, {}, {:#06X}", reg(rt), reg(rs), imm)
    }
}

instruction_struct!(SLL);

impl Instruction for SLL {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let shift = (opcode >> 6) & 0x1F;

        cpu.regs.gpr[rd(opcode)] = cpu.regs.gpr[rt(opcode)] << shift;

        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        let (rd, rt, sa) = (rd(opcode), rt(opcode), sa(opcode));

        if rd == 0 && rt == 0 && sa == 0 {
            "NOP".to_string()
        } else {
            format!("SLL {}, {}, {}", reg(rd), reg(rt), sa)
        }
    }
}

instruction_struct!(SLTU);

impl Instruction for SLTU {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[rd(opcode)] = (cpu.regs.gpr[rs(opcode)] < cpu.regs.gpr[rt(opcode)]) as u64;

        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        format!(
            "SLTU {}, {}, {}",
            reg(rd(opcode)),
            reg(rs(opcode)),
            reg(rt(opcode))
        )
    }
}

instruction_struct!(SW);

impl Instruction for SW {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        let offset = imm16(opcode) as i16 as i32 as u32;
        let rt = rt(opcode);
        let base = rs(opcode); // TODO weird, impl base

        let addr = (cpu.regs.gpr[base] as u32).wrapping_add(offset);
        cpu.write(addr, cpu.regs.gpr[rt] as u32);

        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        let offset = imm16(opcode) as i16;
        let base = rs(opcode);
        let rt = rt(opcode);

        format!("SW {}, {:#06X}({})", reg(rt), offset, reg(base))
    }
}

instruction_struct!(XOR);

impl Instruction for XOR {
    fn execute(&self, cpu: &mut CPU, opcode: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[rd(opcode)] = cpu.regs.gpr[rs(opcode)] ^ cpu.regs.gpr[rt(opcode)];

        None
    }

    fn disassemble(&self, _cpu: &CPU, opcode: u32) -> String {
        format!(
            "XOR {}, {}, {}",
            reg(rd(opcode)),
            reg(rs(opcode)),
            reg(rt(opcode))
        )
    }
}

// Helpers to extract fields from the opcode

fn rs(opcode: u32) -> usize {
    ((opcode >> 21) & 0x1F) as usize
}

fn rt(opcode: u32) -> usize {
    ((opcode >> 16) & 0x1F) as usize
}

fn rd(opcode: u32) -> usize {
    ((opcode >> 11) & 0x1F) as usize
}

fn sa(opcode: u32) -> u32 {
    (opcode >> 6) & 0x1F
}

fn imm16(opcode: u32) -> u16 {
    (opcode & 0xFFFF) as u16
}

// TODO rm???
fn imm16_sext_u64(opcode: u32) -> u64 {
    (imm16(opcode) as i16 as i32) as u64
}

fn reg(i: usize) -> String {
    Registers::gpr_name(i).to_string()
}
