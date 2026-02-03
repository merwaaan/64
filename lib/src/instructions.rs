#![allow(clippy::upper_case_acronyms)]

use crate::cpu::CPU;
use crate::registers::Registers;

/// Result of a branch/jump: target PC to use after the delay slot.
#[derive(Clone, Copy, Debug)]
pub struct DelayedBranching(pub u32);
// TODO impl delay slot skip here?

/// Instruction trait.
pub trait Instruction {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching>;
    fn disassemble(&self, cpu: &CPU, op: u32) -> String;
}

/// Returns the instruction for the given op
pub fn decode(op: u32) -> &'static dyn Instruction {
    let op_top = op >> 26; // TODO rename

    match op_top {
        // Special group
        0x00 => match op & 0x3F {
            0x00 => &SLL_,
            0x02 => &SRL_,
            0x04 => &SLLV_,
            0x06 => &SRLV_,
            0x08 => &JR_,
            0x09 => &JALR_,
            0x10 => &MFHI_,
            0x12 => &MFLO_,
            0x18 => &MULT_,
            0x19 => &MULTU_,
            0x20 => &ADD_,
            0x21 => &ADDU_,
            0x22 => &SUB_,
            0x23 => &SUBU_,
            0x24 => &AND_,
            0x25 => &OR_,
            0x26 => &XOR_,
            0x2A => &SLT_,
            0x2B => &SLTU_,
            _ => &UNKNOWN_,
        },
        // Regimm group
        0x01 => match op & 0x1F_0000 {
            0x110000 => &BGEZAL_,
            _ => &UNKNOWN_,
        },
        // COP1 group
        0x11 => match op & 0x3E0_0000 {
            0x40_0000 => &CFC1_,
            0xC0_0000 => &CTC1_,
            _ => &UNKNOWN_,
        },
        0x03 => &JAL_,
        0x04 => &BEQ_,
        0x05 => &BNE_,
        0x06 => &BLEZ_,
        0x07 => &BGTZ_,
        0x08 => &ADDI_,
        0x09 => &ADDIU_,
        0x0A => &SLTI_,
        0x0B => &SLTIU_,
        0x0C => &ANDI_,
        0x0D => &ORI_,
        0x0E => &XORI_,
        0x0F => &LUI_,
        0x14 => &BEQL_,
        0x15 => &BNEL_,
        0x16 => &BLEZL_,
        0x23 => &LW_,
        0x2B => &SW_,
        0x10 => &COP0_,
        0x2C => &SDL_,
        0x2D => &SDR_,
        0x2F => &CACHE_,
        0x39 => &SWC1_, // TODO generalize?
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
    fn execute(&self, _cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        panic!("Unknown opcode: {:08X}", op)
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("<UNKNOWN {:08X}>", op)
    }
}

instruction_struct!(ADD);

impl Instruction for ADD {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[rd(op)] = cpu.regs.gpr[rs(op)].wrapping_add(cpu.regs.gpr[rt(op)]);

        None

        // TODO overflow rules
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("ADD {}, {}, {}", reg(rd(op)), reg(rs(op)), reg(rt(op)))
    }
}

instruction_struct!(ADDI);

impl Instruction for ADDI {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[rt(op)] = cpu.regs.gpr[rs(op)].wrapping_add(imm16(op) as i16 as u32);

        None

        // TODO overflow rules
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("ADDI {}, {}, {:#06X}", rt(op), rs(op), imm16(op))
    }
}

instruction_struct!(ADDIU);

impl Instruction for ADDIU {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        let imm = (imm16(op) as i16 as i32) as u32;
        let rt = rt(op);
        let rs = rs(op);

        cpu.regs.gpr[rt] = cpu.regs.gpr[rs].wrapping_add(imm);

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("ADDIU {}, {}, {:#06X}", reg(rt(op)), reg(rs(op)), imm16(op))
    }
}

instruction_struct!(ADDU);

impl Instruction for ADDU {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[rd(op)] = cpu.regs.gpr[rs(op)].wrapping_add(cpu.regs.gpr[rt(op)]);

        None

        // TODO no overflow exception
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("ADDU {}, {}, {}", reg(rd(op)), reg(rs(op)), reg(rt(op)))
    }
}

instruction_struct!(AND);

impl Instruction for AND {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[rd(op)] = cpu.regs.gpr[rs(op)] & cpu.regs.gpr[rt(op)];

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("AND {}, {}, {}", reg(rd(op)), reg(rs(op)), reg(rt(op)))
    }
}

instruction_struct!(ANDI);

impl Instruction for ANDI {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        let imm = imm16(op) as u32;
        let rt = rt(op);
        let rs = rs(op);

        cpu.regs.gpr[rt] = cpu.regs.gpr[rs] & imm;

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        let imm = imm16(op);
        let rt = rt(op);
        let rs = rs(op);

        format!("ANDI {}, {}, {:#06X}", reg(rt), reg(rs), imm)
    }
}

instruction_struct!(BEQ);

impl Instruction for BEQ {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        if cpu.regs.gpr[rs(op)] == cpu.regs.gpr[rt(op)] {
            let future_pc = cpu.regs.pc.wrapping_add(4).wrapping_add(branch_offset(op));

            Some(DelayedBranching(future_pc))
        } else {
            None
        }
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!(
            "BEQ {}, {}, {:#06X}",
            reg(rs(op)),
            reg(rt(op)),
            branch_offset(op)
        )
    }
}

instruction_struct!(BEQL);

impl Instruction for BEQL {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        if cpu.regs.gpr[rs(op)] == cpu.regs.gpr[rt(op)] {
            let future_pc = cpu.regs.pc.wrapping_add(4).wrapping_add(branch_offset(op));

            Some(DelayedBranching(future_pc))
        } else {
            // Skip the delay slot
            cpu.regs.pc = cpu.regs.pc.wrapping_add(4);

            None
        }
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!(
            "BEQL {}, {}, {:#06X}",
            reg(rs(op)),
            reg(rt(op)),
            branch_offset(op)
        )
    }
}

instruction_struct!(BGEZAL);

impl Instruction for BGEZAL {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[31] = cpu.regs.pc.wrapping_add(8);

        if (cpu.regs.gpr[rs(op)] as i32) >= 0 {
            let future_pc = cpu.regs.pc.wrapping_add(4).wrapping_add(branch_offset(op));

            Some(DelayedBranching(future_pc))
        } else {
            None
        }
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("BGEZAL {}, {:#06X}", reg(rs(op)), branch_offset(op))
    }
}

instruction_struct!(BGTZ);

impl Instruction for BGTZ {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        if (cpu.regs.gpr[rs(op)] as i32) > 0 {
            let future_pc = cpu.regs.pc.wrapping_add(4).wrapping_add(branch_offset(op));

            Some(DelayedBranching(future_pc))
        } else {
            None
        }
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("BGTZ {}, {:#06X}", reg(rs(op)), branch_offset(op))
    }
}

fn branch_offset(op: u32) -> u32 {
    (imm16(op) as i16 as i32 as u32) << 2
}

instruction_struct!(BLEZ);

impl Instruction for BLEZ {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        if (cpu.regs.gpr[rs(op)] as i32) <= 0 {
            let future_pc = cpu.regs.pc.wrapping_add(4).wrapping_add(branch_offset(op));

            Some(DelayedBranching(future_pc))
        } else {
            None
        }
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("BLEZ {}, {:#06X}", reg(rs(op)), branch_offset(op))
    }
}

instruction_struct!(BLEZL);

impl Instruction for BLEZL {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        if (cpu.regs.gpr[rs(op)] as i32) <= 0 {
            let future_pc = cpu.regs.pc.wrapping_add(4).wrapping_add(branch_offset(op));

            Some(DelayedBranching(future_pc))
        } else {
            // Skip the delay slot
            cpu.regs.pc = cpu.regs.pc.wrapping_add(4);

            None
        }
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("BLEZL {}, {:#06X}", reg(rs(op)), branch_offset(op))
    }
}

instruction_struct!(BNE);

impl Instruction for BNE {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        let rs = rs(op);
        let rt = rt(op);

        if cpu.regs.gpr[rs] != cpu.regs.gpr[rt] {
            let future_pc = cpu.regs.pc.wrapping_add(4).wrapping_add(branch_offset(op));

            Some(DelayedBranching(future_pc))
        } else {
            None
        }
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!(
            "BNE {}, {}, {:#X}",
            reg(rs(op)),
            reg(rt(op)),
            branch_offset(op)
        )
    }
}

instruction_struct!(BNEL);

impl Instruction for BNEL {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        let rs = rs(op);
        let rt = rt(op);

        if cpu.regs.gpr[rs] != cpu.regs.gpr[rt] {
            let future_pc = cpu.regs.pc.wrapping_add(4).wrapping_add(branch_offset(op));

            Some(DelayedBranching(future_pc))
        } else {
            // Skip the delay slot
            cpu.regs.pc = cpu.regs.pc.wrapping_add(4);

            None
        }
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!(
            "BNEL {}, {}, {:#X}",
            reg(rs(op)),
            reg(rt(op)),
            branch_offset(op)
        )
    }
}

instruction_struct!(CACHE);

impl Instruction for CACHE {
    fn execute(&self, _cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        log::warn!("CACHE {:08X}", op);
        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        let sub = (op >> 16) & 0x1F;
        let base = rs(op);

        format!("CACHE {}, {}({})", sub, imm16(op), reg(base))
    }
}

instruction_struct!(CFC1);

impl Instruction for CFC1 {
    fn execute(&self, _cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        log::warn!("CFC1 {:08X}", op);
        // TODO cpu.regs.gpr[rt(op)] = cpu.regs.fpr[rd(op)] as u32;

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("CFC1 {}, {}", reg(rt(op)), regf(rd(op)))
    }
}

instruction_struct!(COP0);

impl Instruction for COP0 {
    fn execute(&self, _cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        let sub = (op >> 21) & 0x1F;

        match sub {
            _ => log::warn!("MTC0 {:08X}", op),
        }

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        let sub = (op >> 21) & 0x1F;

        // TODO

        format!("<COP0 {:02x}>", sub)
    }
}

instruction_struct!(CTC1);

impl Instruction for CTC1 {
    fn execute(&self, _cpu: &mut CPU, _op: u32) -> Option<DelayedBranching> {
        // TODO cpu.regs.gpr[rt(op)] = cpu.regs.fpr[rd(op)] as u32;

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        // TODO
        format!("CTC1 {}, {}", reg(rt(op)), regf(rd(op)))
    }
}

instruction_struct!(JAL);

impl JAL {
    fn target(pc: u32, op: u32) -> u32 {
        let hi = pc.wrapping_add(4) & 0xF000_0000;
        let lo = (op & 0x03FF_FFFF) << 2;
        hi | lo
    }
}

impl Instruction for JAL {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[31] = cpu.regs.pc.wrapping_add(8);

        Some(DelayedBranching(JAL::target(cpu.regs.pc, op)))
    }

    // TODO cpu doesn't necessarily have the correct PC! just pass the PC?
    fn disassemble(&self, cpu: &CPU, op: u32) -> String {
        format!("JAL {:#06X}", JAL::target(cpu.regs.pc, op))
    }
}

instruction_struct!(JALR);

impl Instruction for JALR {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[rd(op)] = cpu.regs.pc.wrapping_add(8);

        Some(DelayedBranching(cpu.regs.gpr[rs(op)]))
    }

    // TODO cpu doesn't necessarily have the correct PC! just pass the PC?
    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("JALR {}, {}", reg(rd(op)), reg(rs(op)))
    }
}

instruction_struct!(JR);

impl Instruction for JR {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        let target = cpu.regs.gpr[rs(op)];

        Some(DelayedBranching(target))
    }

    // TODO cpu doesn't necessarily have the correct PC! just pass the PC?
    fn disassemble(&self, cpu: &CPU, op: u32) -> String {
        let rs = rs(op);
        let addr = cpu.regs.gpr[rs] as u32;

        format!("JR {}={:#06X}", reg(rs), addr)
    }
}

instruction_struct!(LUI);

impl Instruction for LUI {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[rt(op)] = (imm16(op) as u32) << 16;

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("LUI {}, {:#04X}", reg(rt(op)), imm16(op))
    }
}

instruction_struct!(LW);

impl Instruction for LW {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        let offset = imm16(op) as i16 as i32 as u32;

        let addr = cpu.regs.gpr[rs(op)].wrapping_add(offset);

        cpu.regs.gpr[rt(op)] = cpu.read(addr);

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        let offset = imm16(op) as i16;

        format!("LW {}, {:#06X}({})", reg(rt(op)), offset, reg(rs(op)))
    }
}

instruction_struct!(MFHI);

impl Instruction for MFHI {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[rd(op)] = cpu.regs.mult_hi;

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("MFHI {}", reg(rd(op)),)
    }
}

instruction_struct!(MFLO);

impl Instruction for MFLO {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[rd(op)] = cpu.regs.mult_lo;

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("MFLO {}", reg(rd(op)),)
    }
}

instruction_struct!(MULT);

impl Instruction for MULT {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        let rt = rt(op);
        let rs = rs(op);

        let result = (cpu.regs.gpr[rs] as i32 as i64).wrapping_mul(cpu.regs.gpr[rt] as i32 as i64);

        cpu.regs.mult_hi = (result >> 32) as u32; // TODO 64 -> sign extend res
        cpu.regs.mult_lo = result as u32;

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        let rt = rt(op);
        let rs = rs(op);

        format!("MULT {}, {}", reg(rs), reg(rt))
    }
}

instruction_struct!(MULTU);

impl Instruction for MULTU {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        let rt = rt(op);
        let rs = rs(op);

        let result = (cpu.regs.gpr[rs] as u64) * (cpu.regs.gpr[rt] as u64);

        cpu.regs.mult_hi = (result >> 32) as u32;
        cpu.regs.mult_lo = (result & 0xFFFFFFFF) as u32;

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        let rt = rt(op);
        let rs = rs(op);

        format!("MULTU {}, {}", reg(rs), reg(rt))
    }
}

instruction_struct!(OR);

impl Instruction for OR {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[rd(op)] = cpu.regs.gpr[rs(op)] | cpu.regs.gpr[rt(op)];

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("OR {}, {}, {}", reg(rd(op)), reg(rs(op)), reg(rt(op)))
    }
}

instruction_struct!(ORI);

impl Instruction for ORI {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        let imm = imm16(op) as u32;
        let rt = rt(op);
        let rs = rs(op);

        cpu.regs.gpr[rt] = cpu.regs.gpr[rs] | imm;

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        let imm = imm16(op);
        let rt = rt(op);
        let rs = rs(op);

        format!("ORI {}, {}, {:#06X}", reg(rt), reg(rs), imm)
    }
}

// TODO debug lol
// TODO 64
instruction_struct!(SDL);

impl Instruction for SDL {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        let value = cpu.regs.gpr[rt(op)];

        let offset = imm16(op) as i16 as i32 as u32;
        let base = cpu.regs.gpr[base(op)];
        let addr = base + offset;

        let byte_offset = addr & 7;

        for i in 0..8 - byte_offset {
            let byte_addr = addr + i;
            let byte = (value >> (7 - i)) as u8;

            cpu.write8(byte_addr, byte);
        }

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("SDL {}, {:#06X}({})", reg(rt(op)), imm16(op), reg(base(op)))
    }
}

// TODO debug lol
// TODO 64
instruction_struct!(SDR);

impl Instruction for SDR {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        let value = cpu.regs.gpr[rt(op)];

        let offset = imm16(op) as i16 as i32 as u32;
        let base = cpu.regs.gpr[base(op)];
        let addr = base + offset;

        let byte_offset = addr & 7;

        for i in 0..=byte_offset {
            let byte_addr = addr + i;
            let byte = (value >> (byte_offset + i)) as u8;

            cpu.write8(byte_addr, byte);
        }

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("SDR {}, {:#06X}({})", reg(rt(op)), imm16(op), reg(base(op)))
    }
}

instruction_struct!(SLL);

impl Instruction for SLL {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        let shift = (op >> 6) & 0x1F;

        cpu.regs.gpr[rd(op)] = cpu.regs.gpr[rt(op)] << shift;

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        let (rd, rt, sa) = (rd(op), rt(op), sa(op));

        if rd == 0 && rt == 0 && sa == 0 {
            "NOP".to_string()
        } else {
            format!("SLL {}, {}, {}", reg(rd), reg(rt), sa)
        }
    }
}

instruction_struct!(SLLV);

impl Instruction for SLLV {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        let shift = cpu.regs.gpr[rs(op)] & 0x1F;

        cpu.regs.gpr[rd(op)] = cpu.regs.gpr[rt(op)] << shift;

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("SLLV {}, {}, {}", reg(rd(op)), reg(rt(op)), reg(rs(op)))
    }
}

instruction_struct!(SLT);

impl Instruction for SLT {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[rd(op)] =
            ((cpu.regs.gpr[rs(op)] as i32) < (cpu.regs.gpr[rt(op)] as i32)) as u32;

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("SLT {}, {}, {}", reg(rd(op)), reg(rs(op)), reg(rt(op)))
    }
}

instruction_struct!(SLTI);

impl Instruction for SLTI {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[rt(op)] = ((cpu.regs.gpr[rs(op)] as i32) < (imm16(op) as i16 as i32)) as u32;

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("SLTI {}, {}, {:#06X}", reg(rt(op)), reg(rs(op)), imm16(op))
    }
}

instruction_struct!(SLTIU);

impl Instruction for SLTIU {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[rt(op)] = ((cpu.regs.gpr[rs(op)]) < (imm16(op) as i16 as i32 as u32)) as u32;

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("SLTIU {}, {}, {:#06X}", reg(rt(op)), reg(rs(op)), imm16(op))
    }
}

instruction_struct!(SLTU);

impl Instruction for SLTU {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        let rs = rs(op);
        let rt = rt(op);
        let rd = rd(op);

        cpu.regs.gpr[rd] = (cpu.regs.gpr[rs] < cpu.regs.gpr[rt]) as u32;

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("SLTU {}, {}, {}", reg(rd(op)), reg(rs(op)), reg(rt(op)))
    }
}

instruction_struct!(SRL);

impl Instruction for SRL {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        let shift = (op >> 6) & 0x1F;

        cpu.regs.gpr[rd(op)] = cpu.regs.gpr[rt(op)] >> shift;

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        let shift = (op >> 6) & 0x1F;

        format!("SRL {}, {}, {}", reg(rd(op)), reg(rt(op)), shift)
    }
}

instruction_struct!(SRLV);

impl Instruction for SRLV {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        let shift = cpu.regs.gpr[rs(op)] & 0x1F;

        cpu.regs.gpr[rd(op)] = cpu.regs.gpr[rt(op)] >> shift;

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("SRLV {}, {}, {}", reg(rd(op)), reg(rt(op)), reg(rs(op)))
    }
}

instruction_struct!(SUB);

impl Instruction for SUB {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[rd(op)] = cpu.regs.gpr[rs(op)].wrapping_sub(cpu.regs.gpr[rt(op)]);

        None

        // TODO overflow rules
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("SUB {}, {}, {}", reg(rd(op)), reg(rs(op)), reg(rt(op)))
    }
}

instruction_struct!(SUBU);

impl Instruction for SUBU {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[rd(op)] = cpu.regs.gpr[rs(op)].wrapping_sub(cpu.regs.gpr[rt(op)]);

        None

        // TODO no overflow exception
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("SUBU {}, {}, {}", reg(rd(op)), reg(rs(op)), reg(rt(op)))
    }
}

instruction_struct!(SW);

impl Instruction for SW {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        let addr = (cpu.regs.gpr[base(op)] as u32).wrapping_add(imm16(op) as i16 as i32 as u32);

        cpu.write(addr, cpu.regs.gpr[rt(op)] as u32);

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("SW {}, {:#06X}({})", reg(rt(op)), imm16(op), reg(rs(op)))
    }
}

instruction_struct!(SWC1);

impl Instruction for SWC1 {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        let addr = (cpu.regs.gpr[base(op)] as u32).wrapping_add(imm16(op) as i16 as i32 as u32);

        // TODO!
        cpu.write(addr, 0);

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!(
            "SWC1 {}, {:#06X}({})",
            reg(rt(op)),
            imm16(op),
            reg(base(op))
        )
    }
}

instruction_struct!(XOR);

impl Instruction for XOR {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[rd(op)] = cpu.regs.gpr[rs(op)] ^ cpu.regs.gpr[rt(op)];

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("XOR {}, {}, {}", reg(rd(op)), reg(rs(op)), reg(rt(op)))
    }
}

instruction_struct!(XORI);

impl Instruction for XORI {
    fn execute(&self, cpu: &mut CPU, op: u32) -> Option<DelayedBranching> {
        cpu.regs.gpr[rt(op)] = cpu.regs.gpr[rs(op)] ^ imm16(op) as u32;

        None
    }

    fn disassemble(&self, _cpu: &CPU, op: u32) -> String {
        format!("XORI {}, {}, {:#06X}", reg(rt(op)), reg(rs(op)), imm16(op))
    }
}

// Helpers to extract fields from the op

fn base(op: u32) -> usize {
    ((op >> 21) & 0x1F) as usize
}

fn rs(op: u32) -> usize {
    ((op >> 21) & 0x1F) as usize
}

fn rt(op: u32) -> usize {
    ((op >> 16) & 0x1F) as usize
}

fn rd(op: u32) -> usize {
    ((op >> 11) & 0x1F) as usize
}

fn sa(op: u32) -> u32 {
    (op >> 6) & 0x1F
}

fn imm16(op: u32) -> u16 {
    op as u16
}

fn reg(i: usize) -> &'static str {
    Registers::gpr_name(i)
}

fn regf(i: usize) -> &'static str {
    Registers::fpr_name(i)
}
