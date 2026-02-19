//! CPU (and COP1) instruction implementations.

use super::{DelayedBranching, Disassembly, Instruction, Opcode, System};

use crate::data::Data;
use crate::instruction_struct;
use crate::instructions::UNKNOWN_;

pub fn decode_special(opcode: Opcode) -> Option<&'static dyn Instruction> {
    debug_assert_eq!(opcode.group(), 0x00);

    let instruction: &'static dyn Instruction = match opcode.0 & 0x3F {
        0x00 => &SLL_,
        0x02 => &SRL_,
        0x03 => &SRA_,
        0x04 => &SLLV_,
        0x06 => &SRLV_,
        0x07 => &SRAV_,
        0x08 => &JR_,
        0x09 => &JALR_,
        0x0D => &BREAK_,
        0x0F => &SYNC_,
        0x10 => &MFHI_,
        0x11 => &MTHI_,
        0x12 => &MFLO_,
        0x13 => &MTLO_,
        0x14 => &DSLLV_,
        0x16 => &DSRLV_,
        0x17 => &DSRAV_,
        0x18 => &MULT_,
        0x19 => &MULTU_,
        0x1A => &DIV_,
        0x1B => &DIVU_,
        0x1C => &DMULT_,
        0x1D => &DMULTU_,
        0x1E => &DDIV_,
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
        0x2D => &DADDU_,
        0x2E => &DSUB_,
        0x2F => &DSUBU_,
        0x38 => &DSLL_,
        0x3A => &DSRL_,
        0x3B => &DSRA_,
        0x3C => &DSLL32_,
        0x3E => &DSRL32_,
        0x3F => &DSRA32_,
        _ => &UNKNOWN_,
    };

    if std::ptr::eq(instruction, &UNKNOWN_) {
        None
    } else {
        Some(instruction)
    }
}

pub fn decode_regimm(opcode: Opcode) -> Option<&'static dyn Instruction> {
    debug_assert_eq!(opcode.group(), 0x01);

    let instruction: &'static dyn Instruction = match opcode.0 & 0x1F_0000 {
        0x00_0000 => &BLTZ_,
        0x01_0000 => &BGEZ_,
        0x02_0000 => &BLTZL_,
        0x03_0000 => &BGEZL_,
        0x10_0000 => &BLTZAL_,
        0x11_0000 => &BGEZAL_,
        _ => &UNKNOWN_,
    };

    if std::ptr::eq(instruction, &UNKNOWN_) {
        None
    } else {
        Some(instruction)
    }
}

pub fn decode_standard(opcode: Opcode) -> Option<&'static dyn Instruction> {
    let instruction: &'static dyn Instruction = match opcode.group() {
        0x02 => &J_,
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
        0x19 => &DADDIU_,
        0x1A => &LDL_,
        0x1B => &LDR_,
        0x20 => &LB_,
        0x21 => &LH_,
        0x22 => &LWL_,
        0x23 => &LW_,
        0x24 => &LBU_,
        0x25 => &LHU_,
        0x26 => &LWR_,
        0x27 => &LWU_,
        0x28 => &SB_,
        0x29 => &SH_,
        0x2A => &SWL_,
        0x2B => &SW_,
        0x2C => &SDL_,
        0x2D => &SDR_,
        0x2E => &SWR_,
        0x2F => &CACHE_,
        0x30 => &LL_,
        0x31 => &LWC1_,
        0x35 => &LDC1_,
        0x37 => &LD_,
        0x38 => &SC_,
        0x39 => &SWC1_, // TODO generalize?
        0x3D => &SDC1_,
        0x3F => &SD_,
        _ => &UNKNOWN_,
    };

    if std::ptr::eq(instruction, &UNKNOWN_) {
        None
    } else {
        Some(instruction)
    }
}

instruction_struct!(ADD);

impl Instruction for ADD {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set(op.rsv(s).wrapping_add(op.rtv(s)));

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("ADD {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
    }
}

instruction_struct!(ADDI);

impl Instruction for ADDI {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rt()].set(op.rsv(s).wrapping_add(op.imm16() as i16 as u32));

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
        s.cpu.regs.gpr[op.rt()].set(op.rsv(s).wrapping_add(imm));

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
        s.cpu.regs.gpr[op.rd()].set(op.rsv(s).wrapping_add(op.rtv(s)));

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("ADDU {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
    }
}

instruction_struct!(AND);

impl Instruction for AND {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set64(op.rsv64(s) & op.rtv64(s));

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("AND {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
    }
}

instruction_struct!(ANDI);

impl Instruction for ANDI {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rt()].set64(op.rsv64(s) & (op.imm16() as u64));

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
        if op.rsv(s) == op.rtv(s) {
            Some(DelayedBranching(op.branch_target(s)))
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
        if op.rsv(s) == op.rtv(s) {
            Some(DelayedBranching(op.branch_target(s)))
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

instruction_struct!(BGEZ);

impl Instruction for BGEZ {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        if (op.rsv(s) as i32) >= 0 {
            Some(DelayedBranching(op.branch_target(s)))
        } else {
            None
        }
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("BGEZ {}, {:#06X}", op.rsn(), op.branch_offset()))
    }
}

instruction_struct!(BGEZL);

impl Instruction for BGEZL {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        if (op.rsv(s) as i32) >= 0 {
            Some(DelayedBranching(op.branch_target(s)))
        } else {
            s.cpu.regs.pc = s.cpu.regs.pc.wrapping_add(4);
            None
        }
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("BGEZ {}, {:#06X}", op.rsn(), op.branch_offset()))
    }
}

instruction_struct!(BGEZAL);

impl Instruction for BGEZAL {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        // Read before linking (matters when rs == 31)
        let rs = op.rsv(s);

        s.cpu.regs.gpr[31].set(s.cpu.regs.pc.wrapping_add(8));

        if (rs as i32) >= 0 {
            Some(DelayedBranching(op.branch_target(s)))
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
        if (op.rsv(s) as i32) > 0 {
            Some(DelayedBranching(op.branch_target(s)))
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
        if (op.rsv(s) as i32) <= 0 {
            Some(DelayedBranching(op.branch_target(s)))
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
        if (op.rsv(s) as i32) <= 0 {
            Some(DelayedBranching(op.branch_target(s)))
        } else {
            s.cpu.regs.pc = s.cpu.regs.pc.wrapping_add(4);
            None
        }
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("BLEZL {}, {:#06X}", op.rsn(), op.branch_offset()))
    }
}

instruction_struct!(BLTZ);

impl Instruction for BLTZ {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        if (op.rsv(s) as i32) < 0 {
            Some(DelayedBranching(op.branch_target(s)))
        } else {
            None
        }
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("BLTZ {}, {:#06X}", op.rsn(), op.branch_offset()))
    }
}

instruction_struct!(BLTZAL);

impl Instruction for BLTZAL {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        // Read before linking (matters when rs == 31)
        let rs = op.rsv(s);

        s.cpu.regs.gpr[31].set(s.cpu.regs.pc.wrapping_add(8));

        if (rs as i32) < 0 {
            Some(DelayedBranching(op.branch_target(s)))
        } else {
            None
        }
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("BLTZAL {}, {:#06X}", op.rsn(), op.branch_offset()))
    }
}

instruction_struct!(BLTZL);

impl Instruction for BLTZL {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        if (op.rsv(s) as i32) < 0 {
            Some(DelayedBranching(op.branch_target(s)))
        } else {
            s.cpu.regs.pc = s.cpu.regs.pc.wrapping_add(4);
            None
        }
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("BLTZL {}, {:#06X}", op.rsn(), op.branch_offset()))
    }
}

instruction_struct!(BNE);

impl Instruction for BNE {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        if op.rsv(s) != op.rtv(s) {
            Some(DelayedBranching(op.branch_target(s)))
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
        if op.rsv(s) != op.rtv(s) {
            Some(DelayedBranching(op.branch_target(s)))
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
instruction_struct!(BREAK);

impl Instruction for BREAK {
    fn execute(&self, s: &mut System, _op: Opcode) -> Option<DelayedBranching> {
        panic!("BREAK at {:08X}", s.cpu.regs.pc);
    }

    fn disassemble(&self, _s: &System, _op: Opcode) -> Disassembly {
        Disassembly::new("BREAK".to_string())
    }
}

instruction_struct!(CACHE);

impl Instruction for CACHE {
    fn execute(&self, _s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        //TODO log::debug!("CACHE {:08X}", op.0);
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

instruction_struct!(DADDIU);

impl Instruction for DADDIU {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let data = op.imm16() as i16 as i64 as u64;

        s.cpu.regs.gpr[op.rt()].set64(op.rsv64(s).wrapping_add(data));

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DADDIU {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
    }
}

instruction_struct!(DADDU);

impl Instruction for DADDU {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set64(op.rsv64(s).wrapping_add(op.rtv64(s)));

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DADDU {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
    }
}

// TODO div by zero?

instruction_struct!(DIV);

impl Instruction for DIV {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let rsvs = s.cpu.regs.gpr[op.rs()].get() as i32;
        let rtvs = s.cpu.regs.gpr[op.rt()].get() as i32;

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

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DIV {}, {}", op.rsn(), op.rtn()))
    }
}

instruction_struct!(DIVU);

impl Instruction for DIVU {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let rsv = s.cpu.regs.gpr[op.rs()].get();
        let rtv = s.cpu.regs.gpr[op.rt()].get();

        if rtv == 0 {
            s.cpu.regs.mult_hi.set(rsv as u32);
            s.cpu.regs.mult_lo.set(u32::MAX);
        } else {
            s.cpu.regs.mult_hi.set((rsv).overflowing_rem(rtv).0);
            s.cpu.regs.mult_lo.set((rsv).overflowing_div(rtv).0);
        }

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DIVU {}, {}", op.rsn(), op.rtn()))
    }
}

instruction_struct!(DDIV);

impl Instruction for DDIV {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let rsv = s.cpu.regs.gpr[op.rs()].get64() as i64;
        let rtv = s.cpu.regs.gpr[op.rt()].get64() as i64;

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

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DDIV {}, {}", op.rsn(), op.rtn()))
    }
}

instruction_struct!(DDIVU);

impl Instruction for DDIVU {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let rsv = s.cpu.regs.gpr[op.rs()].get64();
        let rtv = s.cpu.regs.gpr[op.rt()].get64();

        if rtv == 0 {
            s.cpu.regs.mult_hi.set64(rsv as u64);
            s.cpu.regs.mult_lo.set64(u64::MAX);
        } else {
            s.cpu.regs.mult_hi.set64((rsv).overflowing_rem(rtv).0);
            s.cpu.regs.mult_lo.set64((rsv).overflowing_div(rtv).0);

            // println!(
            //     "DDIVU {:X}, {:X}, {:X}, {:X}",
            //     rsv,
            //     rtv,
            //     (rsv).overflowing_div(rtv).0,
            //     (rsv).overflowing_rem(rtv).0
            // );

            // println!(
            //     "mult_hi: {:X}, mult_lo: {:X}",
            //     s.cpu.regs.mult_hi.get64(),
            //     s.cpu.regs.mult_lo.get64()
            // );
        }

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DDIVU {}, {}", op.rsn(), op.rtn()))
    }
}

instruction_struct!(DMULT);

impl Instruction for DMULT {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let result = (op.rsv64(s) as i64 as i128) * (op.rtv64(s) as i64 as i128);

        s.cpu.regs.mult_hi.set64((result >> 64) as u64);
        s.cpu.regs.mult_lo.set64(result as u64);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DMULTU {}, {}", op.rsn(), op.rtn()))
    }
}

instruction_struct!(DMULTU);

impl Instruction for DMULTU {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let result = (op.rsv64(s) as u128) * (op.rtv64(s) as u128);

        s.cpu.regs.mult_hi.set64((result >> 64) as u64);
        s.cpu.regs.mult_lo.set64(result as u64);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DMULTU {}, {}", op.rsn(), op.rtn()))
    }
}

instruction_struct!(DSLL);

impl Instruction for DSLL {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let data = op.rtv64(s) << op.shift();

        s.cpu.regs.gpr[op.rd()].set64(data);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DSLL {}, {}, {}", op.rdn(), op.rtn(), op.shift()))
    }
}

instruction_struct!(DSLL32);

impl Instruction for DSLL32 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let data = op.rtv64(s) << (op.shift() + 32);

        s.cpu.regs.gpr[op.rd()].set64(data);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DSLL32 {}, {}, {}", op.rdn(), op.rtn(), op.shift()))
    }
}

instruction_struct!(DSLLV);

impl Instruction for DSLLV {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let data = op.rtv64(s) << (op.rsv(s) & 0x3F);

        s.cpu.regs.gpr[op.rd()].set64(data);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DSLLV {}, {}, {}", op.rdn(), op.rtn(), op.rsn()))
    }
}

instruction_struct!(DSRA);

impl Instruction for DSRA {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let data = (op.rtv64(s) as i64 >> op.shift()) as u64;

        s.cpu.regs.gpr[op.rd()].set64(data);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DSRA {}, {}, {}", op.rdn(), op.rtn(), op.shift()))
    }
}

instruction_struct!(DSRA32);

impl Instruction for DSRA32 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let data = (op.rtv64(s) as i64 >> (op.shift() + 32)) as u64;

        s.cpu.regs.gpr[op.rd()].set64(data);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DSRA32 {}, {}, {}", op.rdn(), op.rtn(), op.shift()))
    }
}

instruction_struct!(DSRAV);

impl Instruction for DSRAV {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let data = ((op.rtv64(s) as i64) >> (op.rsv(s) & 0x3F)) as u64;

        s.cpu.regs.gpr[op.rd()].set64(data);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DSRAV {}, {}, {}", op.rdn(), op.rtn(), op.rsn()))
    }
}

instruction_struct!(DSRL);

impl Instruction for DSRL {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let data = op.rtv64(s) >> op.shift();

        s.cpu.regs.gpr[op.rd()].set64(data);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DSRL {}, {}, {}", op.rdn(), op.rtn(), op.shift()))
    }
}

instruction_struct!(DSRL32);

impl Instruction for DSRL32 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let data = op.rtv64(s) >> (op.shift() + 32);

        s.cpu.regs.gpr[op.rd()].set64(data);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DSRL32 {}, {}, {}", op.rdn(), op.rtn(), op.shift()))
    }
}

instruction_struct!(DSRLV);

impl Instruction for DSRLV {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let data = op.rtv64(s) >> (op.rsv(s) & 0x3F);

        s.cpu.regs.gpr[op.rd()].set64(data);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DSRLV {}, {}, {}", op.rdn(), op.rtn(), op.rsn()))
    }
}

instruction_struct!(DSUB);

impl Instruction for DSUB {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set64(op.rsv64(s).wrapping_sub(op.rtv64(s)));

        // TODO exception

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DSUB {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
    }
}

instruction_struct!(DSUBU);

impl Instruction for DSUBU {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set64(op.rsv64(s).wrapping_sub(op.rtv64(s)));

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DSUBU {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
    }
}

instruction_struct!(J);

impl J {
    fn target(pc: u32, op: Opcode) -> u32 {
        let hi = pc.wrapping_add(4) & 0xF000_0000;
        let lo = (op.0 & 0x03FF_FFFF) << 2;
        hi | lo
    }
}

impl Instruction for J {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        Some(DelayedBranching(J::target(s.cpu.regs.pc, op)))
    }

    // TODO cpu doesn't necessarily have the correct PC! just pass the PC?
    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("J {:#06X}", J::target(s.cpu.regs.pc, op)))
    }
}

instruction_struct!(JAL);

impl Instruction for JAL {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[31].set(s.cpu.regs.pc.wrapping_add(8));

        Some(DelayedBranching(J::target(s.cpu.regs.pc, op)))
    }

    // TODO cpu doesn't necessarily have the correct PC! just pass the PC?
    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("JAL {:#06X}", J::target(s.cpu.regs.pc, op)))
    }
}

instruction_struct!(JALR);

impl Instruction for JALR {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        // Read before linking (matters when rd == rs)
        let target = op.rsv(s);

        s.cpu.regs.gpr[op.rd()].set(s.cpu.regs.pc.wrapping_add(8));

        Some(DelayedBranching(target))
    }

    // TODO cpu doesn't necessarily have the correct PC! just pass the PC?
    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "JALR {}, {}={:#06X}",
            op.rdn(),
            op.rsn(),
            op.rsv(s)
        ))
    }
}

instruction_struct!(JR);

impl Instruction for JR {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        Some(DelayedBranching(op.rsv(s)))
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("JR {}={:#06X}", op.rsn(), op.rsv(s)))
    }
}

instruction_struct!(LB);

impl Instruction for LB {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op.offset_addr(s);

        let data = s.read::<u8>(addr) as i8 as i32 as u32;

        s.cpu.regs.gpr[op.rt()].set(data);

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "LB {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(op.offset_addr(s))
    }
}

instruction_struct!(LBU);

impl Instruction for LBU {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op.offset_addr(s);

        let data = s.read::<u8>(addr) as u32;

        s.cpu.regs.gpr[op.rt()].set(data);

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "LBU {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(op.offset_addr(s))
    }
}

instruction_struct!(LD);

impl Instruction for LD {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op.offset_addr(s);

        s.cpu.regs.gpr[op.rt()].set64(s.read::<u64>(addr));

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "LD {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(op.offset_addr(s))
    }
}
// TODO mvoe down
instruction_struct!(LDC1);

impl Instruction for LDC1 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let data = s.read::<u64>(op.offset_addr(s));

        if s.cop0.f_64() {
            s.cpu.regs.fpr[op.rt()].set64(data as u64);
        } else {
            s.cpu.regs.fpr[op.rt()].set(data as u32);
            s.cpu.regs.fpr[op.rt() + 1].set((data >> 32) as u32);
        }

        // TODO exceptions
        // TODO COP1 enabled?

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("LDC1 {}, {}({})", op.rtn(), op.imm16(), op.basen()))
    }
}

instruction_struct!(LDL);

impl Instruction for LDL {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op.offset_addr(s);
        let base = addr & !7;
        let offset = addr & 7;

        let mut dword = s.read::<u64>(base);

        if offset != 0 {
            dword <<= offset * 8;
            dword |= op.rtv64(s) & !(u64::MAX << (8 * offset));
        }

        s.cpu.regs.gpr[op.rt()].set64(dword);

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "LDL {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(op.offset_addr(s))
    }
}

instruction_struct!(LDR);

impl Instruction for LDR {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op.offset_addr(s);
        let base = addr & !7;
        let offset = addr & 7;

        let mut dword = s.read::<u64>(base);

        if offset != 7 {
            dword >>= (7 - offset) * 8;
            dword |= op.rtv64(s) & (u64::MAX << (8 * (offset + 1)));
        }

        s.cpu.regs.gpr[op.rt()].set64(dword);

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "LDR {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(op.offset_addr(s))
    }
}

instruction_struct!(LH);

impl Instruction for LH {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op.offset_addr(s);

        let data = s.read::<u16>(addr) as i16 as i32 as u32;

        s.cpu.regs.gpr[op.rt()].set(data);

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "LH {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(op.offset_addr(s))
    }
}

instruction_struct!(LHU);

// TODOM LHU @ 802efaa4 not working!
impl Instruction for LHU {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op.offset_addr(s);

        let data = s.read::<u16>(addr).to_u32();

        s.cpu.regs.gpr[op.rt()].set(data);

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "LHU {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(op.offset_addr(s))
    }
}
instruction_struct!(LL);

impl Instruction for LL {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op.offset_addr(s);

        s.cpu.regs.load_linked_bit = true;
        s.cpu.regs.load_linked_addr = addr;

        s.cpu.regs.gpr[op.rt()].set(s.read(addr));

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "LL {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(op.offset_addr(s))
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
        let addr = op.offset_addr(s);

        s.cpu.regs.gpr[op.rt()].set(s.read(addr));

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "LW {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(op.offset_addr(s))
    }
}
// TODO mvoe down
instruction_struct!(LWC1);

impl Instruction for LWC1 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op.offset_addr(s);
        let data = s.read::<u32>(addr);

        if s.cop0.f_64() {
            s.cpu.regs.fpr[op.rt()].set64(data as u64);
        } else {
            s.cpu.regs.fpr[op.rt() & !1].set64(data as u64);
        }

        // TODO exceptions
        // TODO COP1 enabled?

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("LWC1 {}, {}({})", op.rtn(), op.imm16(), op.basen()))
    }
}

instruction_struct!(LWL);

impl Instruction for LWL {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op.offset_addr(s);
        let addr_base = addr & !3;
        let addr_offset = addr & 3;

        let data = s.read::<u32>(addr_base);

        let word = if addr_offset == 0 {
            data
        } else {
            let mut word = s.cpu.regs.gpr[op.rt()].get();
            word &= 0xFFFF_FFFF >> (32 - 8 * addr_offset);
            word |= data << (8 * addr_offset);
            word
        };

        s.cpu.regs.gpr[op.rt()].set(word);

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "LWL {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(op.offset_addr(s))
    }
}

// TODO move partial shift stuff to helpers!

instruction_struct!(LWR);

impl Instruction for LWR {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op.offset_addr(s);

        let addr_base = addr & !3;
        let addr_offset = addr & 3;

        let data = s.read::<u32>(addr_base);

        let word = if addr_offset == 3 {
            data
        } else {
            let mut word = s.cpu.regs.gpr[op.rt()].get();
            word &= !(0xFFFF_FFFF >> (24 - 8 * addr_offset));
            word |= data >> (24 - 8 * addr_offset);
            word
        };

        s.cpu.regs.gpr[op.rt()].set(word);

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "LWR {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(op.offset_addr(s))
    }
}

instruction_struct!(LWU);

impl Instruction for LWU {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op.offset_addr(s);

        s.cpu.regs.gpr[op.rt()].set64(s.read::<u32>(addr) as u64);

        // TODO exception?

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "LWU {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(op.offset_addr(s))
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
        let result = (op.rsv(s) as i32 as i64).wrapping_mul(op.rtv(s) as i32 as i64);

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
        let result = (op.rsv(s) as u64) * (op.rtv(s) as u64);

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
        s.cpu.regs.gpr[op.rd()].set64(!(op.rsv64(s) | op.rtv64(s)));
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("NOR {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
    }
}

instruction_struct!(OR);

impl Instruction for OR {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set64(op.rsv64(s) | op.rtv64(s));
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("OR {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
    }
}

instruction_struct!(ORI);

impl Instruction for ORI {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rt()].set64(op.rsv64(s) | op.imm16() as u64);
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

instruction_struct!(SB);

impl Instruction for SB {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op.offset_addr(s);
        let data = op.rtv(s) as u8;

        s.write(addr, data);

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "SB {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(op.offset_addr(s))
    }
}

instruction_struct!(SC);

impl Instruction for SC {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rt()].set(s.cpu.regs.load_linked_bit as u32);

        if s.cpu.regs.load_linked_bit {
            let addr = op.offset_addr(s);
            s.write(addr, op.rtv(s));
        }

        s.cpu.regs.load_linked_bit = false;

        // TODO impl effects: ERET/write to addr/link addr changed

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "SC {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.basen()
        ))
        .with_address_hint(op.offset_addr(s))
    }
}

instruction_struct!(SD);

impl Instruction for SD {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op.offset_addr(s);

        s.write(addr, s.cpu.regs.gpr[op.rt()].get64());

        // TODO addr exception?

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "SD {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(op.offset_addr(s))
    }
}

instruction_struct!(SDC1);

impl Instruction for SDC1 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op.offset_addr(s);

        if s.cop0.f_64() {
            s.write(addr, s.cpu.regs.fpr[op.rt()].get64());
        } else {
            s.write(addr + 4, s.cpu.regs.fpr[op.rt() + 1].get());
            s.write(addr, s.cpu.regs.fpr[op.rt()].get());
        }

        // TODO addr exception?

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "SDC1 {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(op.offset_addr(s))
    }
}

instruction_struct!(SDL);

impl Instruction for SDL {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op.offset_addr(s);

        let base = addr & !7;
        let offset = addr & 7;

        let dword = if offset == 0 {
            op.rtv64(s)
        } else {
            let mut dword = s.read::<u64>(base);
            dword &= 0xFFFFFFFF_FFFFFFFF << (64 - 8 * offset);
            dword |= op.rtv64(s) >> (8 * offset);
            dword
        };

        s.write::<u64>(base, dword);

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

instruction_struct!(SDR);

impl Instruction for SDR {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op.offset_addr(s);

        let base = addr & !7;
        let offset = addr & 7;

        let dword = if offset == 7 {
            op.rtv64(s)
        } else {
            let mut dword = s.read::<u64>(base);
            dword &= 0xFFFFFFFF_FFFFFFFF >> (8 * (offset + 1));
            dword |= op.rtv64(s) << (56 - 8 * offset);
            dword
        };

        s.write(base, dword);

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
        let addr = op.offset_addr(s);
        let data = op.rtv(s) as u16;

        s.write(addr, data);

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "SH {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(op.offset_addr(s))
    }
}

instruction_struct!(SLL);

impl Instruction for SLL {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set(op.rtv(s) << op.shift());

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
        s.cpu.regs.gpr[op.rd()].set(op.rtv(s) << (op.rsv(s) & 0x1F));
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("SLLV {}, {}, {}", op.rdn(), op.rtn(), op.rsn()))
    }
}

instruction_struct!(SLT);

impl Instruction for SLT {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set(((op.rsv(s) as i32) < (op.rtv(s) as i32)) as u32);
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("SLT {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
    }
}

instruction_struct!(SLTI);

impl Instruction for SLTI {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rt()].set(((op.rsv(s) as i32) < (op.imm16() as i16 as i32)) as u32);
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

        s.cpu.regs.gpr[op.rt()].set((op.rsv(s) < imm) as u32);

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
        s.cpu.regs.gpr[op.rd()].set((op.rsv(s) < op.rtv(s)) as u32);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("SLTU {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
    }
}

instruction_struct!(SRA);

impl Instruction for SRA {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set((op.rtv(s) as i32 >> op.shift()) as u32);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("SRA {}, {}, {}", op.rdn(), op.rtn(), op.shift()))
    }
}

instruction_struct!(SRAV);

impl Instruction for SRAV {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let data = ((op.rtv64(s) as i64) >> (op.rsv(s) & 0x1F)) as u64;

        s.cpu.regs.gpr[op.rd()].set64(data);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("SRAV {}, {}, {}", op.rdn(), op.rtn(), op.rsn()))
    }
}

instruction_struct!(SRL);

impl Instruction for SRL {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set(op.rtv(s) >> op.shift());

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("SRL {}, {}, {}", op.rdn(), op.rtn(), op.shift()))
    }
}

instruction_struct!(SRLV);

impl Instruction for SRLV {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set(op.rtv(s) >> (op.rsv(s) & 0x1F));

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("SRLV {}, {}, {}", op.rdn(), op.rtn(), op.rsn()))
    }
}

instruction_struct!(SUB);

impl Instruction for SUB {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set(op.rsv(s).wrapping_sub(op.rtv(s)));

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("SUB {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
    }
}

instruction_struct!(SUBU);

impl Instruction for SUBU {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set(op.rsv(s).wrapping_sub(op.rtv(s)));
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("SUBU {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
    }
}

instruction_struct!(SW);

impl Instruction for SW {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op.offset_addr(s);

        s.write(addr, op.rtv(s));

        // TODO addr exception if 2bits != 0?

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "SW {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(op.offset_addr(s))
    }
}

instruction_struct!(SWC1);

impl Instruction for SWC1 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op.offset_addr(s);

        if s.cop0.f_64() {
            s.write(addr, s.cpu.regs.fpr[op.rt()].get());
        } else {
            s.write(addr, s.cpu.regs.fpr[op.rt() & !1].get());
        }

        // TODO exceptions
        // TODO COP1 enabled?

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

instruction_struct!(SYNC);

impl Instruction for SYNC {
    fn execute(&self, _s: &mut System, _op: Opcode) -> Option<DelayedBranching> {
        None
    }

    fn disassemble(&self, _s: &System, _op: Opcode) -> Disassembly {
        Disassembly::new("SYNC".to_string())
    }
}

instruction_struct!(SWL);

impl Instruction for SWL {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op.offset_addr(s);
        let addr_base = addr & !3;
        let addr_offset = addr & 3;

        let word = if addr_offset == 0 {
            op.rtv(s)
        } else {
            let mut word = s.read::<u32>(addr_base);
            word &= 0xFFFF_FFFF << (32 - 8 * addr_offset);
            word |= op.rtv(s) >> (8 * addr_offset);
            word
        };

        s.write(addr_base, word);

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "SWL {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(op.offset_addr(s))
    }
}

instruction_struct!(SWR);

impl Instruction for SWR {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        let addr = op.offset_addr(s);

        let base = addr & !3;
        let offset = addr & 3;

        let word = if offset == 3 {
            op.rtv(s)
        } else {
            let mut word = s.read::<u32>(base);
            word &= 0xFFFF_FFFF >> (8 * (offset + 1));
            word |= op.rtv(s) << (24 - 8 * offset);
            word
        };

        s.write(base, word);

        None
    }

    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "SWR {}, {:#06X}({})",
            op.rtn(),
            op.imm16(),
            op.rsn()
        ))
        .with_address_hint(op.offset_addr(s))
    }
}

instruction_struct!(XOR);

impl Instruction for XOR {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rd()].set64(op.rsv64(s) ^ op.rtv64(s));
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("XOR {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
    }
}

instruction_struct!(XORI);

impl Instruction for XORI {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rt()].set64(op.rsv64(s) ^ op.imm16() as u64);
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
