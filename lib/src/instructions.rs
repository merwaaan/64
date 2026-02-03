#![allow(clippy::upper_case_acronyms)]

use crate::cpu::CPU;
use crate::data::Data;
use crate::map::address_info;
use crate::registers::Registers;
use crate::system::System;

#[derive(Clone, Copy)]
pub struct Opcode(pub u32);

impl Opcode {
    fn group(&self) -> u32 {
        self.0 >> 26
    }

    // x -> register index
    // xv -> register value
    // xn -> register name
    // x0n -> COP0 register name

    fn base(&self) -> usize {
        ((self.0 >> 21) & 0x1F) as usize
    }

    fn basev(&self, cpu: &CPU) -> u32 {
        cpu.regs.gpr[self.base()].get()
    }

    fn basen(&self) -> &'static str {
        Registers::gpr_name(self.base())
    }

    fn rs(&self) -> usize {
        ((self.0 >> 21) & 0x1F) as usize
    }

    fn rsv(&self, cpu: &CPU) -> u32 {
        cpu.regs.gpr[self.rs()].get()
    }

    fn rsn(&self) -> &'static str {
        Registers::gpr_name(self.rs())
    }

    fn rt(&self) -> usize {
        ((self.0 >> 16) & 0x1F) as usize
    }

    fn rtv(&self, cpu: &CPU) -> u32 {
        cpu.regs.gpr[self.rt()].get()
    }

    fn rtn(&self) -> &'static str {
        Registers::gpr_name(self.rt())
    }

    fn rt0n(&self) -> &'static str {
        Registers::cop0_name(self.rt())
    }

    fn rd(&self) -> usize {
        ((self.0 >> 11) & 0x1F) as usize
    }

    fn rdn(&self) -> &'static str {
        Registers::gpr_name(self.rd())
    }

    fn rd0n(&self) -> &'static str {
        Registers::cop0_name(self.rd())
    }

    fn shift(&self) -> u32 {
        (self.0 >> 6) & 0x1F
    }

    fn imm16(&self) -> u16 {
        self.0 as u16
    }

    fn branch_offset(&self) -> u32 {
        (self.imm16() as i16 as i32 as u32) << 2
    }

    fn branch_target(&self, cpu: &CPU) -> u32 {
        cpu.regs
            .pc
            .wrapping_add(4)
            .wrapping_add(self.branch_offset())
    }
}

/// Result of a branch/jump: target PC to use after the delay slot.
#[derive(Clone, Copy, Debug)]
pub struct DelayedBranching(pub u32);
// TODO impl delay slot skip here?

pub struct Disassembly {
    pub mnemonics: String,
    pub hint: Option<String>,
}

impl Disassembly {
    pub fn new(mnemonics: String) -> Self {
        Self {
            mnemonics,
            hint: None,
        }
    }

    pub fn with_hint(self, hint: String) -> Self {
        Self {
            hint: Some(hint),
            ..self
        }
    }

    pub fn with_address_hint(self, addr: u32) -> Self {
        if let Some(hint) = address_info(addr) {
            Self {
                hint: Some(hint.to_string()),
                ..self
            }
        } else {
            self
        }
    }
}

/// Instruction trait.
pub trait Instruction {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching>;
    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly;
}

/// Returns the instruction for the given op
pub fn decode(opcode: Opcode) -> &'static dyn Instruction {
    match opcode.group() {
        // Special group
        0x00 => match opcode.0 & 0x3F {
            0x00 => &SLL_,
            0x02 => &SRL_,
            0x03 => &SRA_,
            0x04 => &SLLV_,
            0x06 => &SRLV_,
            0x08 => &JR_,
            0x09 => &JALR_,
            0x10 => &MFHI_,
            0x11 => &MTHI_,
            0x12 => &MFLO_,
            0x13 => &MTLO_,
            0x18 => &MULT_,
            0x19 => &MULTU_,
            0x1D => &DMULTU_,
            0x1F => &DDIVU_,
            0x20 => &ADD_,
            0x21 => &ADDU_,
            0x22 => &SUB_,
            0x23 => &SUBU_,
            0x24 => &AND_,
            0x25 => &OR_,
            0x26 => &XOR_,
            0x27 => &NOR_,
            0x2A => &SLT_,
            0x2B => &SLTU_,
            0x3C => &DSLL32_,
            0x3F => &DSRA32_,
            _ => &UNKNOWN_,
        },
        // Regimm group
        0x01 => match opcode.0 & 0x1F_0000 {
            0x110000 => &BGEZAL_,
            _ => &UNKNOWN_,
        },
        // COP0 group
        0x10 => match opcode.0 & 0x3E0_0000 {
            0x000_0000 => &MFC0_,
            0x080_0000 => &MTC0_,
            0x200_0000 => &TLBWI_,
            _ => &UNKNOWN_,
        },
        // COP1 group
        0x11 => match opcode.0 & 0x3E0_0000 {
            0x040_0000 => &CFC1_,
            0x0C0_0000 => &CTC1_,
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
        0x21 => &LH_,
        0x23 => &LW_,
        0x25 => &LHU_,
        0x29 => &SH_,
        0x2B => &SW_,
        0x2C => &SDL_,
        0x2D => &SDR_,
        0x2F => &CACHE_,
        0x30 => &LL_,
        0x37 => &LD_,
        0x38 => &SC_,
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
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        unimplemented!("Unknown opcode {:08X} @ {:08X}", op.0, s.cpu.regs.pc)
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("<UNKNOWN {:08X}>", op.0))
    }
}

instruction_struct!(ADD);

impl Instruction for ADD {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set(op.rsv(&s.cpu).wrapping_add(op.rtv(&s.cpu)));
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("ADD {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
    }
}

instruction_struct!(ADDI);

impl Instruction for ADDI {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rt()].set(op.rsv(&s.cpu).wrapping_add(op.imm16() as i16 as u32));
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "ADDI {}, {}, {:#06X}",
            op.rtn(),
            op.rsn(),
            op.imm16()
        ))
    }
}

instruction_struct!(ADDIU);

impl Instruction for ADDIU {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let imm = (op.imm16() as i16 as i32) as u32;
        s.cpu.regs.gpr[op.rt()].set(op.rsv(&s.cpu).wrapping_add(imm));
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "ADDIU {}, {}, {:#06X}",
            op.rtn(),
            op.rsn(),
            op.imm16()
        ))
    }
}

instruction_struct!(ADDU);

impl Instruction for ADDU {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set(op.rsv(&s.cpu).wrapping_add(op.rtv(&s.cpu)));
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("ADDU {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
    }
}

instruction_struct!(AND);

impl Instruction for AND {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set(op.rsv(&s.cpu) & op.rtv(&s.cpu));
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("AND {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
    }
}

instruction_struct!(ANDI);

impl Instruction for ANDI {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rt()].set(op.rsv(&s.cpu) & op.imm16() as u32);
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "ANDI {}, {}, {:#06X}",
            op.rtn(),
            op.rsn(),
            op.imm16()
        ))
    }
}

// TODO sahre branching offset func!

instruction_struct!(BEQ);

impl Instruction for BEQ {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        if op.rsv(&s.cpu) == op.rtv(&s.cpu) {
            Some(DelayedBranching(op.branch_target(&s.cpu)))
        } else {
            None
        }
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "BEQ {}, {}, {:#06X}",
            op.rsn(),
            op.rtn(),
            op.branch_offset()
        ))
    }
}

instruction_struct!(BEQL);

impl Instruction for BEQL {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        if op.rsv(&s.cpu) == op.rtv(&s.cpu) {
            Some(DelayedBranching(op.branch_target(&s.cpu)))
        } else {
            s.cpu.regs.pc = s.cpu.regs.pc.wrapping_add(4);
            None
        }
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "BEQL {}, {}, {:#06X}",
            op.rsn(),
            op.rtn(),
            op.branch_offset()
        ))
    }
}

instruction_struct!(BGEZAL);

impl Instruction for BGEZAL {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        // Read before linking (matters when rs == 31)
        let rs = op.rsv(&s.cpu);

        s.cpu.regs.gpr[31].set(s.cpu.regs.pc.wrapping_add(8));

        if (rs as i32) >= 0 {
            Some(DelayedBranching(op.branch_target(&s.cpu)))
        } else {
            None
        }
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("BGEZAL {}, {:#06X}", op.rsn(), op.branch_offset()))
        // TODO cond result?
    }
}

instruction_struct!(BGTZ);

impl Instruction for BGTZ {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        if (op.rsv(&s.cpu) as i32) > 0 {
            Some(DelayedBranching(op.branch_target(&s.cpu)))
        } else {
            None
        }
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("BGTZ {}, {:#06X}", op.rsn(), op.branch_offset()))
    }
}

instruction_struct!(BLEZ);

impl Instruction for BLEZ {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        if (op.rsv(&s.cpu) as i32) <= 0 {
            Some(DelayedBranching(op.branch_target(&s.cpu)))
        } else {
            None
        }
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("BLEZ {}, {:#06X}", op.rsn(), op.branch_offset()))
    }
}

instruction_struct!(BLEZL);

impl Instruction for BLEZL {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        if (op.rsv(&s.cpu) as i32) <= 0 {
            Some(DelayedBranching(op.branch_target(&s.cpu)))
        } else {
            s.cpu.regs.pc = s.cpu.regs.pc.wrapping_add(4);
            None
        }
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("BLEZL {}, {:#06X}", op.rsn(), op.branch_offset()))
    }
}

instruction_struct!(BNE);

impl Instruction for BNE {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        if op.rsv(&s.cpu) != op.rtv(&s.cpu) {
            Some(DelayedBranching(op.branch_target(&s.cpu)))
        } else {
            None
        }
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "BNE {}, {}, {:#X}",
            op.rsn(),
            op.rtn(),
            op.branch_offset()
        ))
    }
}

instruction_struct!(BNEL);

impl Instruction for BNEL {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        if op.rsv(&s.cpu) != op.rtv(&s.cpu) {
            Some(DelayedBranching(op.branch_target(&s.cpu)))
        } else {
            s.cpu.regs.pc = s.cpu.regs.pc.wrapping_add(4);
            None
        }
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "BNEL {}, {}, {:#X}",
            op.rsn(),
            op.rtn(),
            op.branch_offset()
        ))
    }
}

instruction_struct!(CACHE);

impl Instruction for CACHE {
    fn execute(&self, _s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        log::warn!("CACHE {:08X}", op.0);
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "CACHE {}, {}({})",
            op.rtn(),
            op.imm16(),
            op.basen()
        ))
    }
}

instruction_struct!(CFC1);

impl Instruction for CFC1 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rt()].set64(s.cpu.regs.fpr[op.rd()] as u64);
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("CFC1 {}, {}", op.rtn(), regf(op.rd())))
    }
}

instruction_struct!(DDIVU);

impl Instruction for DDIVU {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let rsv = s.cpu.regs.gpr[op.rs()].get64();
        let rtv = s.cpu.regs.gpr[op.rt()].get64();

        let quotient = rsv / rtv;
        let remainder = rsv % rtv;

        s.cpu.regs.mult_hi.set64(remainder);
        s.cpu.regs.mult_lo.set64(quotient);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DDIVU {}, {}", op.rsn(), op.rtn()))
    }
}

instruction_struct!(DMULTU);

impl Instruction for DMULTU {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let result =
            (s.cpu.regs.gpr[op.rs()].get64() as u128) * (s.cpu.regs.gpr[op.rt()].get64() as u128);

        s.cpu.regs.mult_hi.set64((result >> 64) as u64);
        s.cpu.regs.mult_lo.set64(result as u64);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DMULTU {}, {}", op.rsn(), op.rtn()))
    }
}

instruction_struct!(DSLL32);

impl Instruction for DSLL32 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let data = s.cpu.regs.gpr[op.rt()].get64() << (op.shift() + 32);

        s.cpu.regs.gpr[op.rd()].set64(data);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DSLL32 {}, {}, {}", op.rdn(), op.rtn(), op.shift()))
    }
}

instruction_struct!(DSRA32);

impl Instruction for DSRA32 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let data = (s.cpu.regs.gpr[op.rt()].get64() as i64 >> (op.shift() + 32)) as u64;

        s.cpu.regs.gpr[op.rd()].set64(data);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DSRA32 {}, {}, {}", op.rdn(), op.rtn(), op.shift()))
    }
}

instruction_struct!(MTC0);

impl Instruction for MTC0 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let mut data = s.cpu.regs.gpr[op.rt()].get64();

        // TODO cause: only two last bits can be written! move to reg implem
        if op.rd() == 13 {
            data = (data & 3) | (s.cpu.regs.cop0[13].get64() & 0xFFFF_FFFF_FFFF_FFFC);
        }

        s.cpu.regs.cop0[op.rd()].set64(data);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("MTC0 {}, {}", op.rtn(), op.rd0n()))
    }
}

instruction_struct!(MFC0);

impl Instruction for MFC0 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rt()].set64(s.cpu.regs.cop0[op.rd()].get64());

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("MFC0 {}, {}", op.rtn(), op.rd0n()))
    }
}

instruction_struct!(CTC1);

impl Instruction for CTC1 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        // TODO cpu.regs.gpr[op.rt()] = cpu.regs.fpr[op.rd()] as u32;

        s.cpu.regs.fpr[op.rd()] = s.cpu.regs.gpr[op.rt()].get64() as f64;

        // TODO exceptions

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        // TODO
        Disassembly::new(format!("CTC1 {}, {}", op.rtn(), regf(op.rd())))
    }
}

instruction_struct!(JAL);

impl JAL {
    fn target(pc: u32, op: Opcode) -> u32 {
        let hi = pc.wrapping_add(4) & 0xF000_0000;
        let lo = (op.0 & 0x03FF_FFFF) << 2;
        hi | lo
    }
}

impl Instruction for JAL {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[31].set(s.cpu.regs.pc.wrapping_add(8));

        Some(DelayedBranching(JAL::target(s.cpu.regs.pc, op)))
    }

    // TODO cpu doesn't necessarily have the correct PC! just pass the PC?
    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("JAL {:#06X}", JAL::target(s.cpu.regs.pc, op)))
    }
}

instruction_struct!(JALR);

impl Instruction for JALR {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        // Read before linking (matters when rd == rs)
        let target = op.rsv(&s.cpu);

        s.cpu.regs.gpr[op.rd()].set(s.cpu.regs.pc.wrapping_add(8));

        Some(DelayedBranching(target))
    }

    // TODO cpu doesn't necessarily have the correct PC! just pass the PC?
    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "JALR {}, {}={:#06X}",
            op.rdn(),
            op.rsn(),
            op.rsv(&s.cpu)
        ))
    }
}

instruction_struct!(JR);

impl Instruction for JR {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        Some(DelayedBranching(op.rsv(&s.cpu)))
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("JR {}={:#06X}", op.rsn(), op.rsv(&s.cpu)))
    }
}

instruction_struct!(LD);

impl Instruction for LD {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op
            .basev(&s.cpu)
            .wrapping_add(op.imm16() as i16 as i32 as u32);

        s.cpu.regs.gpr[op.rt()].set64(s.read::<u64>(addr));

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        let addr = op
            .basev(&s.cpu)
            .wrapping_add(op.imm16() as i16 as i32 as u32);

        Disassembly::new(format!(
            "LD {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(addr)
    }
}

instruction_struct!(LH);

impl Instruction for LH {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op
            .basev(&s.cpu)
            .wrapping_add(op.imm16() as i16 as i32 as u32);

        let data = s.read::<u16>(addr) as i16 as i32 as u32;

        s.cpu.regs.gpr[op.rt()].set(data);

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        let addr = op
            .basev(&s.cpu)
            .wrapping_add(op.imm16() as i16 as i32 as u32);

        Disassembly::new(format!(
            "LH {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(addr)
    }
}

instruction_struct!(LHU);

// TODOM LHU @ 802efaa4 not working!
impl Instruction for LHU {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op
            .basev(&s.cpu)
            .wrapping_add(op.imm16() as i16 as i32 as u32);

        let data = s.read::<u16>(addr).to_u32();

        s.cpu.regs.gpr[op.rt()].set(data);

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        let addr = op
            .basev(&s.cpu)
            .wrapping_add(op.imm16() as i16 as i32 as u32);

        Disassembly::new(format!(
            "LHU {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(addr)
    }
}
instruction_struct!(LL);

impl Instruction for LL {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op
            .basev(&s.cpu)
            .wrapping_add(op.imm16() as i16 as i32 as u32);

        s.cpu.regs.load_linked_bit = true;
        s.cpu.regs.load_linked_addr = addr;

        s.cpu.regs.gpr[op.rt()].set(s.read(addr));

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        let addr = op
            .basev(&s.cpu)
            .wrapping_add(op.imm16() as i16 as i32 as u32);

        Disassembly::new(format!(
            "LL {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(addr)
    }
}

instruction_struct!(LUI);

impl Instruction for LUI {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rt()].set((op.imm16() as u32) << 16);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("LUI {}, {:#04X}", op.rtn(), op.imm16()))
    }
}

instruction_struct!(LW);

impl Instruction for LW {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op
            .basev(&s.cpu)
            .wrapping_add(op.imm16() as i16 as i32 as u32);

        s.cpu.regs.gpr[op.rt()].set(s.read(addr));

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        let addr = op
            .basev(&s.cpu)
            .wrapping_add(op.imm16() as i16 as i32 as u32);

        Disassembly::new(format!(
            "LW {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(addr)
    }
}

instruction_struct!(MFHI);

impl Instruction for MFHI {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set(s.cpu.regs.mult_hi.get());

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("MFHI {}", op.rdn()))
    }
}

instruction_struct!(MFLO);

impl Instruction for MFLO {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set(s.cpu.regs.mult_lo.get());

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("MFLO {}", op.rdn()))
    }
}

instruction_struct!(MTHI);

impl Instruction for MTHI {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.mult_hi.set64(s.cpu.regs.gpr[op.rs()].get64());

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("MTHI {}", op.rsn()))
    }
}

instruction_struct!(MTLO);

impl Instruction for MTLO {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.mult_lo.set64(s.cpu.regs.gpr[op.rs()].get64());

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("MTLO {}", op.rsn()))
    }
}

instruction_struct!(MULT);

impl Instruction for MULT {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let result = (op.rsv(&s.cpu) as i32 as i64).wrapping_mul(op.rtv(&s.cpu) as i32 as i64);

        s.cpu.regs.mult_hi.set((result >> 32) as u32); // TODO 64 -> sign extend res???
        s.cpu.regs.mult_lo.set(result as u32);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("MULT {}, {}", op.rsn(), op.rtn()))
    }
}

instruction_struct!(MULTU);

impl Instruction for MULTU {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let result = (op.rsv(&s.cpu) as u64) * (op.rtv(&s.cpu) as u64);

        s.cpu.regs.mult_hi.set((result >> 32) as u32);
        s.cpu.regs.mult_lo.set(result as u32);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("MULTU {}, {}", op.rsn(), op.rtn()))
    }
}

instruction_struct!(NOR);

impl Instruction for NOR {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set(!(op.rsv(&s.cpu) | op.rtv(&s.cpu)));
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("NOR {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
    }
}

instruction_struct!(OR);

impl Instruction for OR {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set(op.rsv(&s.cpu) | op.rtv(&s.cpu));
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("OR {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
    }
}

instruction_struct!(ORI);

impl Instruction for ORI {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rt()].set(op.rsv(&s.cpu) | op.imm16() as u32);
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "ORI {}, {}, {:#06X}",
            op.rtn(),
            op.rsn(),
            op.imm16()
        ))
    }
}

instruction_struct!(SC);

impl Instruction for SC {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op
            .basev(&s.cpu)
            .wrapping_add(op.imm16() as i16 as i32 as u32);

        s.cpu.regs.gpr[op.rt()].set(s.cpu.regs.load_linked_bit as u32);

        if s.cpu.regs.load_linked_bit {
            s.write(addr, op.rtv(&s.cpu));
        }

        s.cpu.regs.load_linked_bit = false;

        // TODO impl effects: ERET/write to addr/link addr changed

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        let addr = op
            .basev(&s.cpu)
            .wrapping_add(op.imm16() as i16 as i32 as u32);

        Disassembly::new(format!(
            "SC {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.basen()
        ))
        .with_address_hint(addr)
    }
}

// TODO debug lol
// TODO 64
instruction_struct!(SDL);

impl Instruction for SDL {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let value = op.rtv(&s.cpu);
        let addr = op
            .basev(&s.cpu)
            .wrapping_add(op.imm16() as i16 as i32 as u32);

        let byte_offset = addr & 7;

        for i in 0..8 - byte_offset {
            let byte_addr = addr + i;
            let byte = (value >> (7 - i)) as u8;

            s.write(byte_addr, byte);
        }

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "SDL {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.basen()
        ))
    }
}

// TODO debug lol
// TODO 64
instruction_struct!(SDR);

impl Instruction for SDR {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let value = op.rtv(&s.cpu);
        let addr = op
            .basev(&s.cpu)
            .wrapping_add(op.imm16() as i16 as i32 as u32);

        let byte_offset = addr & 7;

        for i in 0..=byte_offset {
            let byte_addr = addr + i;
            let byte = (value >> (byte_offset + i)) as u8;

            s.write(byte_addr, byte);
        }

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "SDR {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.basen()
        ))
    }
}

instruction_struct!(SH);

impl Instruction for SH {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op
            .basev(&s.cpu)
            .wrapping_add(op.imm16() as i16 as i32 as u32);

        let data = op.rtv(&s.cpu) as u16;

        s.write(addr, data);

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        let addr = op
            .basev(&s.cpu)
            .wrapping_add(op.imm16() as i16 as i32 as u32);

        Disassembly::new(format!(
            "SH {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(addr)
    }
}

instruction_struct!(SLL);

impl Instruction for SLL {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set(op.rtv(&s.cpu) << op.shift());

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        // SLL R0, R0 is NOP

        Disassembly::new(if op.rd() == 0 && op.rt() == 0 {
            "NOP".to_string()
        } else {
            format!("SLL {}, {}, {}", op.rdn(), op.rtn(), op.shift())
        })
    }
}

instruction_struct!(SLLV);

impl Instruction for SLLV {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set(op.rtv(&s.cpu) << (op.rsv(&s.cpu) & 0x1F));
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("SLLV {}, {}, {}", op.rdn(), op.rtn(), op.rsn()))
    }
}

instruction_struct!(SLT);

impl Instruction for SLT {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set(((op.rsv(&s.cpu) as i32) < (op.rtv(&s.cpu) as i32)) as u32);
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("SLT {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
    }
}

instruction_struct!(SLTI);

impl Instruction for SLTI {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rt()].set(((op.rsv(&s.cpu) as i32) < (op.imm16() as i16 as i32)) as u32);
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "SLTI {}, {}, {:#06X}",
            op.rtn(),
            op.rsn(),
            op.imm16()
        ))
    }
}

instruction_struct!(SLTIU);

impl Instruction for SLTIU {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let imm = op.imm16() as i16 as i32 as u32;
        s.cpu.regs.gpr[op.rt()].set((op.rsv(&s.cpu) < imm) as u32);
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "SLTIU {}, {}, {:#06X}",
            op.rtn(),
            op.rsn(),
            op.imm16()
        ))
    }
}

instruction_struct!(SLTU);

impl Instruction for SLTU {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set((op.rsv(&s.cpu) < op.rtv(&s.cpu)) as u32);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("SLTU {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
    }
}

instruction_struct!(SRA);

impl Instruction for SRA {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set((op.rtv(&s.cpu) as i32 >> op.shift()) as u32);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("SRA {}, {}, {}", op.rdn(), op.rtn(), op.shift()))
    }
}

instruction_struct!(SRL);

impl Instruction for SRL {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set(op.rtv(&s.cpu) >> op.shift());

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("SRL {}, {}, {}", op.rdn(), op.rtn(), op.shift()))
    }
}

instruction_struct!(SRLV);

impl Instruction for SRLV {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set(op.rtv(&s.cpu) >> (op.rsv(&s.cpu) & 0x1F));

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("SRLV {}, {}, {}", op.rdn(), op.rtn(), op.rsn()))
    }
}

instruction_struct!(SUB);

impl Instruction for SUB {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set(op.rsv(&s.cpu).wrapping_sub(op.rtv(&s.cpu)));

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("SUB {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
    }
}

instruction_struct!(SUBU);

impl Instruction for SUBU {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set(op.rsv(&s.cpu).wrapping_sub(op.rtv(&s.cpu)));
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("SUBU {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
    }
}

instruction_struct!(SW);

impl Instruction for SW {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op
            .basev(&s.cpu)
            .wrapping_add(op.imm16() as i16 as i32 as u32);
        s.write(addr, op.rtv(&s.cpu));
        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        let addr = op
            .basev(&s.cpu)
            .wrapping_add(op.imm16() as i16 as i32 as u32);

        Disassembly::new(format!(
            "SW {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(addr)
    }
}

instruction_struct!(SWC1);

impl Instruction for SWC1 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op
            .basev(&s.cpu)
            .wrapping_add(op.imm16() as i16 as i32 as u32);
        // TODO!
        s.write(addr, 0u32);
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "SWC1 {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.basen()
        ))
    }
}

instruction_struct!(TLBWI);

impl Instruction for TLBWI {
    fn execute(&self, s: &mut System, _op: Opcode) -> Option<DelayedBranching> {
        log::warn!("TLBWI @ {:08X}", s.cpu.regs.pc);

        None
    }

    fn disassemble(&self, _s: &System, _op: Opcode) -> Disassembly {
        Disassembly::new("TLBWI".to_string())
    }
}

instruction_struct!(XOR);

impl Instruction for XOR {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set(op.rsv(&s.cpu) ^ op.rtv(&s.cpu));
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("XOR {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
    }
}

instruction_struct!(XORI);

impl Instruction for XORI {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rt()].set(op.rsv(&s.cpu) ^ op.imm16() as u32);
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "XORI {}, {}, {:#06X}",
            op.rtn(),
            op.rsn(),
            op.imm16()
        ))
    }
}

// TODO rm?
fn regf(i: usize) -> &'static str {
    Registers::fpr_name(i)
}
