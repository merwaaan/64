use crate::{
    check_aligned, check_cop_usable,
    cpu::{
        instructions::{DecodedInstruction, Disassembly, InstructionEffect, InstructionResult},
        opcode::Opcode,
    },
    exception::Exception,
    inst,
    system::{Address, System},
};

pub fn decode_special(opcode: Opcode) -> Option<DecodedInstruction> {
    debug_assert_eq!(opcode.group(), 0x00);

    Some(match opcode.0 & 0x3F {
        0x00 => inst!(sll),
        0x02 => inst!(srl),
        0x03 => inst!(sra),
        0x04 => inst!(sllv),
        0x06 => inst!(srlv),
        0x07 => inst!(srav),
        0x08 => inst!(jr),
        0x09 => inst!(jalr),
        0x0C => inst!(syscall),
        0x0D => inst!(r#break),
        0x0F => inst!(sync),
        0x10 => inst!(mfhi),
        0x11 => inst!(mthi),
        0x12 => inst!(mflo),
        0x13 => inst!(mtlo),
        0x14 => inst!(dsllv),
        0x16 => inst!(dsrlv),
        0x17 => inst!(dsrav),
        0x18 => inst!(mult),
        0x19 => inst!(multu),
        0x1A => inst!(div),
        0x1B => inst!(divu),
        0x1C => inst!(dmult),
        0x1D => inst!(dmultu),
        0x1E => inst!(ddiv),
        0x1F => inst!(ddivu),
        0x20 => inst!(add),
        0x21 => inst!(addu),
        0x22 => inst!(sub),
        0x23 => inst!(subu),
        0x24 => inst!(and),
        0x25 => inst!(or),
        0x26 => inst!(xor),
        0x27 => inst!(nor),
        0x2A => inst!(slt),
        0x2B => inst!(sltu),
        0x2C => inst!(dadd),
        0x2D => inst!(daddu),
        0x2E => inst!(dsub),
        0x2F => inst!(dsubu),
        0x30 => inst!(tge),
        0x31 => inst!(tgeu),
        0x32 => inst!(tlt),
        0x33 => inst!(tltu),
        0x34 => inst!(teq),
        0x36 => inst!(tne),
        0x38 => inst!(dsll),
        0x3A => inst!(dsrl),
        0x3B => inst!(dsra),
        0x3C => inst!(dsll32),
        0x3E => inst!(dsrl32),
        0x3F => inst!(dsra32),
        _ => return None, // TODO reserved exception?
    })
}

pub fn decode_regimm(opcode: Opcode) -> Option<DecodedInstruction> {
    debug_assert_eq!(opcode.group(), 0x01);

    Some(match opcode.0 & 0x1F_0000 {
        0x00_0000 => inst!(bltz),
        0x01_0000 => inst!(bgez),
        0x02_0000 => inst!(bltzl),
        0x03_0000 => inst!(bgezl),
        0x08_0000 => inst!(tgei),
        0x09_0000 => inst!(tgeiu),
        0x0A_0000 => inst!(tlti),
        0x0B_0000 => inst!(tltiu),
        0x0C_0000 => inst!(teqi),
        0x0E_0000 => inst!(tnei),
        0x10_0000 => inst!(bltzal),
        0x11_0000 => inst!(bgezal),
        0x13_0000 => inst!(bgezall),
        _ => return None, // TODO reserved exception?
    })
}

pub fn decode_standard(opcode: Opcode) -> Option<DecodedInstruction> {
    Some(match opcode.group() {
        0x02 => inst!(j),
        0x03 => inst!(jal),
        0x04 => inst!(beq),
        0x05 => inst!(bne),
        0x06 => inst!(blez),
        0x07 => inst!(bgtz),
        0x08 => inst!(addi),
        0x09 => inst!(addiu),
        0x0A => inst!(slti),
        0x0B => inst!(sltiu),
        0x0C => inst!(andi),
        0x0D => inst!(ori),
        0x0E => inst!(xori),
        0x0F => inst!(lui),
        0x14 => inst!(beql),
        0x15 => inst!(bnel),
        0x16 => inst!(blezl),
        0x17 => inst!(bgtzl),
        0x18 => inst!(daddi),
        0x19 => inst!(daddiu),
        0x1A => inst!(ldl),
        0x1B => inst!(ldr),
        0x20 => inst!(lb),
        0x21 => inst!(lh),
        0x22 => inst!(lwl),
        0x23 => inst!(lw),
        0x24 => inst!(lbu),
        0x25 => inst!(lhu),
        0x26 => inst!(lwr),
        0x27 => inst!(lwu),
        0x28 => inst!(sb),
        0x29 => inst!(sh),
        0x2A => inst!(swl),
        0x2B => inst!(sw),
        0x2C => inst!(sdl),
        0x2D => inst!(sdr),
        0x2E => inst!(swr),
        0x2F => inst!(cache),
        0x30 => inst!(ll),
        0x31 => inst!(lwc1),
        0x34 => inst!(lld),
        0x35 => inst!(ldc1),
        // TODO ldc2, swc2, etc?? or cop2 group???
        0x37 => inst!(ld),
        0x38 => inst!(sc),
        0x39 => inst!(swc1),
        0x3C => inst!(scd),
        0x3D => inst!(sdc1),
        0x3F => inst!(sd),
        _ => return None, // TODO reserved exception?
    })
}

fn add_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let rs = op.rsv(s) as i32;
    let rt = op.rtv(s) as i32;

    match rs.checked_add(rt) {
        Some(result) => {
            s.cpu.regs.gpr[op.rd()].set(result as u32);
            Ok(None)
        }
        None => Err(Exception::ArithmeticOverflow),
    }
}

pub fn add_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("ADD {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
}

fn addi_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let rs = op.rsv(s) as i32;
    let imm = op.imm16() as i16 as i32;

    match rs.checked_add(imm) {
        Some(result) => {
            s.cpu.regs.gpr[op.rt()].set(result as u32);
            Ok(None)
        }
        None => Err(Exception::ArithmeticOverflow),
    }
}

pub fn addi_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "ADDI {}, {}, {:#06X}",
        op.rtn(),
        op.rsn(),
        op.imm16()
    ))
}

fn addiu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let imm = (op.imm16() as i16 as i32) as u32;

    s.cpu.regs.gpr[op.rt()].set(op.rsv(s).wrapping_add(imm));

    Ok(None)
}

pub fn addiu_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "ADDIU {}, {}, {:#06X}",
        op.rtn(),
        op.rsn(),
        op.imm16()
    ))
}

fn addu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.gpr[op.rd()].set(op.rsv(s).wrapping_add(op.rtv(s)));

    Ok(None)
}

pub fn addu_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("ADDU {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
}

fn and_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.gpr[op.rd()].set64(op.rsv64(s) & op.rtv64(s));

    Ok(None)
}

pub fn and_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("AND {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
}

fn andi_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.gpr[op.rt()].set64(op.rsv64(s) & (op.imm16() as u64));

    Ok(None)
}

pub fn andi_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "ANDI {}, {}, {:#06X}",
        op.rtn(),
        op.rsn(),
        op.imm16()
    ))
}

// TODO generic branching?
fn beq_execute(s: &mut System, op: Opcode) -> InstructionResult {
    Ok(Some(InstructionEffect::DelayedBranching(
        if op.rsv64(s) == op.rtv64(s) {
            Some(op.branch_target(s))
        } else {
            None
        },
    )))
}

pub fn beq_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "BEQ {}, {}, {:#06X}",
        op.rsn(),
        op.rtn(),
        op.branch_offset()
    ))
}

fn beql_execute(s: &mut System, op: Opcode) -> InstructionResult {
    if op.rsv64(s) == op.rtv64(s) {
        Ok(Some(InstructionEffect::DelayedBranching(Some(
            op.branch_target(s),
        ))))
    } else {
        // Discard the instruction in the delay slot TODO return special val??
        s.cpu.regs.pc = s.cpu.regs.pc.wrapping_add(4);

        Ok(None)
    }
}

pub fn beql_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "BEQL {}, {}, {:#06X}",
        op.rsn(),
        op.rtn(),
        op.branch_offset()
    ))
}

fn bgez_execute(s: &mut System, op: Opcode) -> InstructionResult {
    Ok(Some(InstructionEffect::DelayedBranching(
        if (op.rsv64(s) as i64) >= 0 {
            Some(op.branch_target(s))
        } else {
            None
        },
    )))
}

pub fn bgez_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("BGEZ {}, {:#06X}", op.rsn(), op.branch_offset()))
}

fn bgezl_execute(s: &mut System, op: Opcode) -> InstructionResult {
    if (op.rsv64(s) as i64) >= 0 {
        Ok(Some(InstructionEffect::DelayedBranching(Some(
            op.branch_target(s),
        ))))
    } else {
        // Discard the instruction in the delay slot TODO return special val??
        s.cpu.regs.pc = s.cpu.regs.pc.wrapping_add(4);

        Ok(None)
    }
}

pub fn bgezl_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("BGEZ {}, {:#06X}", op.rsn(), op.branch_offset()))
}

fn bgezal_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // Read before linking (matters when rs == 31)
    let rs = op.rsv64(s) as i64;

    // The return address is the instruction that follows the delay slot
    s.cpu.regs.gpr[31].set(s.cpu.regs.pc.wrapping_add(8));

    Ok(Some(InstructionEffect::DelayedBranching(if rs >= 0 {
        Some(op.branch_target(s))
    } else {
        None
    })))
}

pub fn bgezal_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("BGEZAL {}, {:#06X}", op.rsn(), op.branch_offset()))
    // TODO cond result?
}

fn bgezall_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // Read before linking (matters when rs == 31)
    let rs = op.rsv64(s) as i64;

    // The return address is the instruction that follows the delay slot
    s.cpu.regs.gpr[31].set(s.cpu.regs.pc.wrapping_add(8));

    if rs >= 0 {
        Ok(Some(InstructionEffect::DelayedBranching(Some(
            op.branch_target(s),
        ))))
    } else {
        // Discard the instruction in the delay slot TODO return special val??
        s.cpu.regs.pc = s.cpu.regs.pc.wrapping_add(4);

        Ok(None)
    }
}

pub fn bgezall_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("BGEZALL {}, {:#06X}", op.rsn(), op.branch_offset()))
    // TODO cond result?
}

fn bgtz_execute(s: &mut System, op: Opcode) -> InstructionResult {
    Ok(Some(InstructionEffect::DelayedBranching(
        if (op.rsv64(s) as i64) > 0 {
            Some(op.branch_target(s))
        } else {
            None
        },
    )))
}

pub fn bgtz_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("BGTZ {}, {:#06X}", op.rsn(), op.branch_offset()))
}

fn bgtzl_execute(s: &mut System, op: Opcode) -> InstructionResult {
    if (op.rsv64(s) as i64) > 0 {
        Ok(Some(InstructionEffect::DelayedBranching(Some(
            op.branch_target(s),
        ))))
    } else {
        // Discard the instruction in the delay slot TODO return special val??
        s.cpu.regs.pc = s.cpu.regs.pc.wrapping_add(4);

        Ok(None)
    }
}

fn bgtzl_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("BGTZL {}, {:#06X}", op.rsn(), op.branch_offset()))
}

fn blez_execute(s: &mut System, op: Opcode) -> InstructionResult {
    Ok(Some(InstructionEffect::DelayedBranching(
        if (op.rsv64(s) as i64) <= 0 {
            Some(op.branch_target(s))
        } else {
            None
        },
    )))
}

pub fn blez_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("BLEZ {}, {:#06X}", op.rsn(), op.branch_offset()))
}

fn blezl_execute(s: &mut System, op: Opcode) -> InstructionResult {
    if (op.rsv64(s) as i64) <= 0 {
        Ok(Some(InstructionEffect::DelayedBranching(Some(
            op.branch_target(s),
        ))))
    } else {
        // Discard the instruction in the delay slot TODO return special val??
        s.cpu.regs.pc = s.cpu.regs.pc.wrapping_add(4);

        Ok(None)
    }
}

pub fn blezl_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("BLEZL {}, {:#06X}", op.rsn(), op.branch_offset()))
}

fn bltz_execute(s: &mut System, op: Opcode) -> InstructionResult {
    Ok(Some(InstructionEffect::DelayedBranching(
        if (op.rsv64(s) as i64) < 0 {
            Some(op.branch_target(s))
        } else {
            None
        },
    )))
}

pub fn bltz_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("BLTZ {}, {:#06X}", op.rsn(), op.branch_offset()))
}

fn bltzal_execute(s: &mut System, op: Opcode) -> InstructionResult {
    // Read before linking (matters when rs == 31)
    let rs = op.rsv64(s) as i64;

    // The return address is the instruction that follows the delay slot
    s.cpu.regs.gpr[31].set(s.cpu.regs.pc.wrapping_add(8));

    Ok(Some(InstructionEffect::DelayedBranching(if rs < 0 {
        Some(op.branch_target(s))
    } else {
        None
    })))
}

pub fn bltzal_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("BLTZAL {}, {:#06X}", op.rsn(), op.branch_offset()))
}

fn bltzl_execute(s: &mut System, op: Opcode) -> InstructionResult {
    if (op.rsv64(s) as i64) < 0 {
        Ok(Some(InstructionEffect::DelayedBranching(Some(
            op.branch_target(s),
        ))))
    } else {
        // Discard the instruction in the delay slot TODO return special val??
        s.cpu.regs.pc = s.cpu.regs.pc.wrapping_add(4);

        Ok(None)
    }
}

pub fn bltzl_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("BLTZL {}, {:#06X}", op.rsn(), op.branch_offset()))
}

fn bne_execute(s: &mut System, op: Opcode) -> InstructionResult {
    Ok(Some(InstructionEffect::DelayedBranching(
        if op.rsv64(s) != op.rtv64(s) {
            Some(op.branch_target(s))
        } else {
            None
        },
    )))
}

pub fn bne_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "BNE {}, {}, {:#X}",
        op.rsn(),
        op.rtn(),
        op.branch_offset()
    ))
}

fn bnel_execute(s: &mut System, op: Opcode) -> InstructionResult {
    if op.rsv64(s) != op.rtv64(s) {
        Ok(Some(InstructionEffect::DelayedBranching(Some(
            op.branch_target(s),
        ))))
    } else {
        // Discard the instruction in the delay slot TODO return special val??
        s.cpu.regs.pc = s.cpu.regs.pc.wrapping_add(4);

        Ok(None)
    }
}

pub fn bnel_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "BNEL {}, {}, {:#X}",
        op.rsn(),
        op.rtn(),
        op.branch_offset()
    ))
}

fn break_execute(_s: &mut System, _op: Opcode) -> InstructionResult {
    Err(Exception::Breakpoint)
}

pub fn break_disassemble(_s: &System, _op: Opcode) -> Disassembly {
    Disassembly::new("BREAK".to_string())
}

fn cache_execute(_s: &mut System, _op: Opcode) -> InstructionResult {
    //TODO log::debug!("CACHE {:08X}", op.0);
    Ok(None)
}

pub fn cache_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "CACHE {}, {}({})",
        op.rtn(),
        op.imm16(),
        op.basen()
    ))
}

fn dadd_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let rs = op.rsv64(s) as i64;
    let rt = op.rtv64(s) as i64;

    match rs.checked_add(rt) {
        Some(result) => {
            s.cpu.regs.gpr[op.rd()].set64(result as u64);
            Ok(None)
        }
        None => Err(Exception::ArithmeticOverflow),
    }
}

pub fn dadd_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DADD {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
}

fn daddi_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let rs = op.rsv64(s) as i64;
    let imm = op.imm16() as i16 as i64;

    match rs.checked_add(imm) {
        Some(result) => {
            s.cpu.regs.gpr[op.rt()].set64(result as u64);
            Ok(None)
        }
        None => Err(Exception::ArithmeticOverflow),
    }
}

pub fn daddi_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DADDI {}, {}, {}", op.rtn(), op.rsn(), op.imm16()))
}

fn daddiu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let res = op.rsv64(s).wrapping_add(op.imm16() as i16 as i64 as u64);

    s.cpu.regs.gpr[op.rt()].set64(res);

    Ok(None)
}

pub fn daddiu_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "DADDIU {}, {}, {:#06X}",
        op.rtn(),
        op.rsn(),
        op.imm16()
    ))
}

fn daddu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.gpr[op.rd()].set64(op.rsv64(s).wrapping_add(op.rtv64(s)));

    Ok(None)
}

pub fn daddu_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DADDU {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
}

// TODO div by zero?

fn div_execute(s: &mut System, op: Opcode) -> InstructionResult {
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

    Ok(None)
}

pub fn div_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DIV {}, {}", op.rsn(), op.rtn()))
}

fn divu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let rsv = s.cpu.regs.gpr[op.rs()].get();
    let rtv = s.cpu.regs.gpr[op.rt()].get();

    if rtv == 0 {
        s.cpu.regs.mult_hi.set(rsv);
        s.cpu.regs.mult_lo.set(u32::MAX);
    } else {
        s.cpu.regs.mult_hi.set((rsv).overflowing_rem(rtv).0);
        s.cpu.regs.mult_lo.set((rsv).overflowing_div(rtv).0);
    }

    Ok(None)
}

pub fn divu_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DIVU {}, {}", op.rsn(), op.rtn()))
}

fn ddiv_execute(s: &mut System, op: Opcode) -> InstructionResult {
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

    Ok(None)
}

pub fn ddiv_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DDIV {}, {}", op.rsn(), op.rtn()))
}

fn ddivu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let rs = op.rsv64(s);
    let rt = op.rtv64(s);

    if rt == 0 {
        s.cpu.regs.mult_hi.set64(rs);
        s.cpu.regs.mult_lo.set64(u64::MAX);
    } else {
        s.cpu.regs.mult_hi.set64((rs).overflowing_rem(rt).0);
        s.cpu.regs.mult_lo.set64((rs).overflowing_div(rt).0);
    }

    Ok(None)
}

pub fn ddivu_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DDIVU {}, {}", op.rsn(), op.rtn()))
}

fn dmult_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let result = (op.rsv64(s) as i64 as i128) * (op.rtv64(s) as i64 as i128);

    s.cpu.regs.mult_hi.set64((result >> 64) as u64);
    s.cpu.regs.mult_lo.set64(result as u64);

    Ok(None)
}

pub fn dmult_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DMULT {}, {}", op.rsn(), op.rtn()))
}

fn dmultu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let result = (op.rsv64(s) as u128) * (op.rtv64(s) as u128);

    s.cpu.regs.mult_hi.set64((result >> 64) as u64);
    s.cpu.regs.mult_lo.set64(result as u64);

    Ok(None)
}

pub fn dmultu_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DMULTU {}, {}", op.rsn(), op.rtn()))
}

fn dsll_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let data = op.rtv64(s) << op.shift();

    s.cpu.regs.gpr[op.rd()].set64(data);

    Ok(None)
}

pub fn dsll_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DSLL {}, {}, {}", op.rdn(), op.rtn(), op.shift()))
}

fn dsll32_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let data = op.rtv64(s) << (op.shift() + 32);

    s.cpu.regs.gpr[op.rd()].set64(data);

    Ok(None)
}

pub fn dsll32_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DSLL32 {}, {}, {}", op.rdn(), op.rtn(), op.shift()))
}

fn dsllv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let data = op.rtv64(s) << (op.rsv(s) & 0x3F);

    s.cpu.regs.gpr[op.rd()].set64(data);

    Ok(None)
}

pub fn dsllv_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DSLLV {}, {}, {}", op.rdn(), op.rtn(), op.rsn()))
}

fn dsra_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let data = (op.rtv64(s) as i64 >> op.shift()) as u64;

    s.cpu.regs.gpr[op.rd()].set64(data);

    Ok(None)
}

pub fn dsra_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DSRA {}, {}, {}", op.rdn(), op.rtn(), op.shift()))
}

fn dsra32_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let data = (op.rtv64(s) as i64 >> (op.shift() + 32)) as u64;

    s.cpu.regs.gpr[op.rd()].set64(data);

    Ok(None)
}

pub fn dsra32_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DSRA32 {}, {}, {}", op.rdn(), op.rtn(), op.shift()))
}

fn dsrav_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let data = ((op.rtv64(s) as i64) >> (op.rsv(s) & 0x3F)) as u64;

    s.cpu.regs.gpr[op.rd()].set64(data);

    Ok(None)
}

pub fn dsrav_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DSRAV {}, {}, {}", op.rdn(), op.rtn(), op.rsn()))
}

fn dsrl_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let data = op.rtv64(s) >> op.shift();

    s.cpu.regs.gpr[op.rd()].set64(data);

    Ok(None)
}

pub fn dsrl_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DSRL {}, {}, {}", op.rdn(), op.rtn(), op.shift()))
}

fn dsrl32_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let data = op.rtv64(s) >> (op.shift() + 32);
    s.cpu.regs.gpr[op.rd()].set64(data);

    Ok(None)
}

pub fn dsrl32_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DSRL32 {}, {}, {}", op.rdn(), op.rtn(), op.shift()))
}

fn dsrlv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let data = op.rtv64(s) >> (op.rsv(s) & 0x3F);
    s.cpu.regs.gpr[op.rd()].set64(data);

    Ok(None)
}

pub fn dsrlv_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DSRLV {}, {}, {}", op.rdn(), op.rtn(), op.rsn()))
}

fn dsub_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let rs = op.rsv64(s) as i64;
    let rt = op.rtv64(s) as i64;

    match rs.checked_sub(rt) {
        Some(result) => {
            s.cpu.regs.gpr[op.rd()].set64(result as u64);
            Ok(None)
        }
        None => Err(Exception::ArithmeticOverflow),
    }
}

pub fn dsub_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DSUB {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
}

fn dsubu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.gpr[op.rd()].set64(op.rsv64(s).wrapping_sub(op.rtv64(s)));

    Ok(None)
}

pub fn dsubu_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DSUBU {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
}

fn j_target(pc: u32, op: Opcode) -> u32 {
    let hi = pc.wrapping_add(4) & 0xF000_0000;
    let lo = (op.0 & 0x03FF_FFFF) << 2;
    hi | lo
}

fn j_execute(s: &mut System, op: Opcode) -> InstructionResult {
    Ok(Some(InstructionEffect::DelayedBranching(Some(j_target(
        s.cpu.regs.pc,
        op,
    )))))
}

pub fn j_disassemble(s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("J {:#06X}", j_target(s.cpu.regs.pc, op)))
}

fn jal_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.gpr[31].set(s.cpu.regs.pc.wrapping_add(8));

    Ok(Some(InstructionEffect::DelayedBranching(Some(j_target(
        s.cpu.regs.pc,
        op,
    )))))
}

pub fn jal_disassemble(s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("JAL {:#06X}", j_target(s.cpu.regs.pc, op)))
}

fn jalr_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let target = op.rsv(s);

    s.cpu.regs.gpr[op.rd()].set(s.cpu.regs.pc.wrapping_add(8));

    Ok(Some(InstructionEffect::DelayedBranching(Some(target))))
}

pub fn jalr_disassemble(s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "JALR {}, {}={:#06X}",
        op.rdn(),
        op.rsn(),
        op.rsv(s)
    ))
}

fn jr_execute(s: &mut System, op: Opcode) -> InstructionResult {
    Ok(Some(InstructionEffect::DelayedBranching(Some(op.rsv(s)))))
}

pub fn jr_disassemble(s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("JR {}={:#06X}", op.rsn(), op.rsv(s)))
}

fn lb_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);
    let data = s.read::<u8>(Address::v(addr))? as i8 as i32 as u32;
    s.cpu.regs.gpr[op.rt()].set(data);

    Ok(None)
}

pub fn lb_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "LB {}, {:#06X}({})",
        op.rtn(),
        op.imm16(),
        op.rsn()
    ))
    //.with_address_hint(op.offset_addr(s))
}

fn lbu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);
    let data = s.read::<u8>(Address::v(addr))?;
    s.cpu.regs.gpr[op.rt()].set(data as u32);

    Ok(None)
}

pub fn lbu_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "LBU {}, {:#06X}({})",
        op.rtn(),
        op.imm16(),
        op.rsn()
    ))
    //.with_address_hint(op.offset_addr(s))
}

fn ld_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);
    check_aligned!(load, addr, 7);

    let data = s.read(Address::v(addr))?;
    s.cpu.regs.gpr[op.rt()].set64(data);

    Ok(None)
}

pub fn ld_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "LD {}, {:#06X}({})",
        op.rtn(),
        op.imm16(),
        op.rsn()
    ))
    //.with_address_hint(op.offset_addr(s))
}

fn ldc1_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    let addr = op.offset_addr(s);
    check_aligned!(load, addr, 7);

    let data = s.read(Address::v(addr))?;
    s.cop1.set64(op.ft(), data, s.cop0.f64());

    Ok(None)
}

pub fn ldc1_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("LDC1 {}, {}({})", op.ftn(), op.imm16(), op.basen()))
}

fn ldl_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);
    let addr_base = addr & !7;
    let addr_offset = addr & 7;

    let mut data = s.read(Address::v(addr_base))?;

    if addr_offset != 0 {
        data <<= addr_offset * 8;
        data |= op.rtv64(s) & !(u64::MAX << (8 * addr_offset));
    }

    s.cpu.regs.gpr[op.rt()].set64(data);

    Ok(None)
}

pub fn ldl_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "LDL {}, {:#06X}({})",
        op.rtn(),
        op.imm16(),
        op.rsn()
    ))
    //.with_address_hint(op.offset_addr(s))
}

fn ldr_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);
    let base = addr & !7;
    let offset = addr & 7;

    let mut data = s.read(Address::v(base))?;

    if offset != 7 {
        data >>= (7 - offset) * 8;
        data |= op.rtv64(s) & (u64::MAX << (8 * (offset + 1)));
    }

    s.cpu.regs.gpr[op.rt()].set64(data);

    Ok(None)
}

pub fn ldr_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "LDR {}, {:#06X}({})",
        op.rtn(),
        op.imm16(),
        op.rsn()
    ))
    //.with_address_hint(op.offset_addr(s))
}

fn lh_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);
    check_aligned!(load, addr, 1);

    let data = s.read::<u16>(Address::v(addr))?;
    s.cpu.regs.gpr[op.rt()].set(data as i16 as i32 as u32);

    Ok(None)
}

pub fn lh_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "LH {}, {:#06X}({})",
        op.rtn(),
        op.imm16(),
        op.rsn()
    ))
    //.with_address_hint(op.offset_addr(s))
}

fn lhu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);
    check_aligned!(load, addr, 1);

    let data = s.read::<u16>(Address::v(addr))?;
    s.cpu.regs.gpr[op.rt()].set(data as u32);

    Ok(None)
}

pub fn lhu_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "LHU {}, {:#06X}({})",
        op.rtn(),
        op.imm16(),
        op.rsn()
    ))
    //.with_address_hint(op.offset_addr(s))
}

fn ll_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);
    check_aligned!(load, addr, 3);

    s.cop0.set_ll_addr(addr);
    s.cpu.regs.load_linked_bit = true;

    let data = s.read(Address::v(addr))?;
    s.cpu.regs.gpr[op.rt()].set(data);

    Ok(None)
}

pub fn ll_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "LL {}, {:#06X}({})",
        op.rtn(),
        op.imm16(),
        op.rsn()
    ))
    //.with_address_hint(op.offset_addr(s))
}

fn lld_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);
    check_aligned!(load, addr, 3);

    s.cop0.set_ll_addr(addr);
    s.cpu.regs.load_linked_bit = true;

    let data = s.read(Address::v(addr))?;
    s.cpu.regs.gpr[op.rt()].set64(data);

    Ok(None)
}

pub fn lld_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "LDD {}, {:#06X}({})",
        op.rtn(),
        op.imm16(),
        op.rsn()
    ))
    //.with_address_hint(op.offset_addr(s))
}

fn lui_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.gpr[op.rt()].set((op.imm16() as u32) << 16);

    Ok(None)
}

pub fn lui_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("LUI {}, {:#04X}", op.rtn(), op.imm16()))
}

fn lw_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);
    check_aligned!(load, addr, 3);

    let data = s.read(Address::v(addr))?;
    s.cpu.regs.gpr[op.rt()].set(data);

    Ok(None)
}

pub fn lw_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "LW {}, {:#06X}({})",
        op.rtn(),
        op.imm16(),
        op.rsn()
    ))
    //.with_address_hint(op.offset_addr(s))
}

fn lwc1_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    let addr = op.offset_addr(s);
    check_aligned!(load, addr, 3);

    let data = s.read(Address::v(addr))?;
    s.cop1.set32(op.rt(), data, s.cop0.f64());

    Ok(None)
}

pub fn lwc1_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("LWC1 {}, {}({})", op.ftn(), op.imm16(), op.basen()))
}

fn lwl_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);
    let addr_base = addr & !3;
    let addr_offset = addr & 3;

    let data = s.read(Address::v(addr_base))?;

    let word = if addr_offset == 0 {
        data
    } else {
        let mut word = s.cpu.regs.gpr[op.rt()].get();
        word &= 0xFFFF_FFFF >> (32 - 8 * addr_offset);
        word |= data << (8 * addr_offset);
        word
    };

    s.cpu.regs.gpr[op.rt()].set(word);

    Ok(None)
}

pub fn lwl_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "LWL {}, {:#06X}({})",
        op.rtn(),
        op.imm16(),
        op.rsn()
    ))
    //.with_address_hint(op.offset_addr(s))
}

// TODO move partial shift stuff to helpers!

fn lwr_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);
    let addr_base = addr & !3;
    let addr_offset = addr & 3;

    let data = s.read(Address::v(addr_base))?;

    let word = if addr_offset == 3 {
        data
    } else {
        let mut word = s.cpu.regs.gpr[op.rt()].get();
        word &= !(0xFFFF_FFFF >> (24 - 8 * addr_offset));
        word |= data >> (24 - 8 * addr_offset);
        word
    };

    s.cpu.regs.gpr[op.rt()].set(word);

    Ok(None)
}

pub fn lwr_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "LWR {}, {:#06X}({})",
        op.rtn(),
        op.imm16(),
        op.rsn()
    ))
    //.with_address_hint(op.offset_addr(s))
}

fn lwu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);
    check_aligned!(load, addr, 3);

    let data = s.read::<u32>(Address::v(addr))?;
    s.cpu.regs.gpr[op.rt()].set64(data as u64);

    Ok(None)
}

pub fn lwu_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "LWU {}, {:#06X}({})",
        op.rtn(),
        op.imm16(),
        op.rsn()
    ))
    //.with_address_hint(op.offset_addr(s))
}

fn mfhi_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.gpr[op.rd()].set64(s.cpu.regs.mult_hi.get64());

    Ok(None)
}

pub fn mfhi_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("MFHI {}", op.rdn()))
}

fn mflo_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.gpr[op.rd()].set64(s.cpu.regs.mult_lo.get64());

    Ok(None)
}

pub fn mflo_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("MFLO {}", op.rdn()))
}

fn mthi_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.mult_hi.set64(op.rsv64(s));

    Ok(None)
}

pub fn mthi_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("MTHI {}", op.rsn()))
}

fn mtlo_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.mult_lo.set64(op.rsv64(s));

    Ok(None)
}

pub fn mtlo_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("MTLO {}", op.rsn()))
}

fn mult_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let result = (op.rsv(s) as i32 as i64).wrapping_mul(op.rtv(s) as i32 as i64);

    s.cpu.regs.mult_hi.set((result >> 32) as u32);
    s.cpu.regs.mult_lo.set(result as u32);

    Ok(None)
}

pub fn mult_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("MULT {}, {}", op.rsn(), op.rtn()))
}

fn multu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let result = (op.rsv(s) as u64) * (op.rtv(s) as u64);

    s.cpu.regs.mult_hi.set((result >> 32) as u32);
    s.cpu.regs.mult_lo.set(result as u32);

    Ok(None)
}

pub fn multu_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("MULTU {}, {}", op.rsn(), op.rtn()))
}

fn nor_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.gpr[op.rd()].set64(!(op.rsv64(s) | op.rtv64(s)));

    Ok(None)
}

pub fn nor_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("NOR {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
}

fn or_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.gpr[op.rd()].set64(op.rsv64(s) | op.rtv64(s));

    Ok(None)
}

pub fn or_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("OR {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
}

fn ori_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.gpr[op.rt()].set64(op.rsv64(s) | op.imm16() as u64);

    Ok(None)
}

pub fn ori_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "ORI {}, {}, {:#06X}",
        op.rtn(),
        op.rsn(),
        op.imm16()
    ))
}

fn sb_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.write(Address::v(op.offset_addr(s)), op.rtv(s) as u8)?;

    Ok(None)
}

pub fn sb_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "SB {}, {:#06X}({})",
        op.rtn(),
        op.imm16(),
        op.rsn()
    ))
    //.with_address_hint(op.offset_addr(s))
}

fn sc_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);
    check_aligned!(store, addr, 3);

    s.cpu.regs.gpr[op.rt()].set(s.cpu.regs.load_linked_bit as u32);

    if s.cpu.regs.load_linked_bit {
        s.write(Address::v(addr), op.rtv(s))?;
    }

    s.cpu.regs.load_linked_bit = false;

    Ok(None)
}

pub fn sc_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "SC {}, {:#06X}({})",
        op.rtn(),
        op.imm16(),
        op.basen()
    ))
    //.with_address_hint(op.offset_addr(s))
}

fn scd_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);
    check_aligned!(store, addr, 3);

    s.cpu.regs.gpr[op.rt()].set64(s.cpu.regs.load_linked_bit as u64);

    if s.cpu.regs.load_linked_bit {
        s.write(Address::v(addr), op.rtv64(s))?;
    }

    s.cpu.regs.load_linked_bit = false;

    Ok(None)
}

pub fn scd_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "SCD {}, {:#06X}({})",
        op.rtn(),
        op.imm16(),
        op.basen()
    ))
    //.with_address_hint(op.offset_addr(s))
}

fn sd_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);
    check_aligned!(store, addr, 7);

    s.write(Address::v(addr), s.cpu.regs.gpr[op.rt()].get64())?;

    Ok(None)
}

pub fn sd_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "SD {}, {:#06X}({})",
        op.rtn(),
        op.imm16(),
        op.rsn()
    ))
    //.with_address_hint(op.offset_addr(s))
}

fn sdc1_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    let addr = op.offset_addr(s);
    check_aligned!(store, addr, 7);

    s.write(Address::v(addr), s.cop1.get64(op.ft(), s.cop0.f64()))?;

    Ok(None)
}

pub fn sdc1_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "SDC1 {}, {:#06X}({})",
        op.ftn(),
        op.imm16(),
        op.basen()
    ))
    //.with_address_hint(op.offset_addr(s))
}

fn sdl_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);
    let base = addr & !7;
    let offset = addr & 7;

    let dword = if offset == 0 {
        op.rtv64(s)
    } else {
        let mut dword = s.peek(Address::v(base)).expect("Invalid address");
        dword &= 0xFFFFFFFF_FFFFFFFF << (64 - 8 * offset);
        dword |= op.rtv64(s) >> (8 * offset);
        dword
    };

    s.write(Address::v(base), dword)?;

    Ok(None)
}

pub fn sdl_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "SDL {}, {:#06X}({})",
        op.rtn(),
        op.imm16(),
        op.basen()
    ))
}

fn sdr_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);
    let base = addr & !7;
    let offset = addr & 7;

    let data = if offset == 7 {
        op.rtv64(s)
    } else {
        let mut dword = s.peek(Address::v(base)).expect("Invalid address");
        dword &= 0xFFFFFFFF_FFFFFFFF >> (8 * (offset + 1));
        dword |= op.rtv64(s) << (56 - 8 * offset);
        dword
    };

    s.write(Address::v(base), data)?;

    Ok(None)
}

pub fn sdr_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "SDR {}, {:#06X}({})",
        op.rtn(),
        op.imm16(),
        op.basen()
    ))
}

fn sh_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);
    check_aligned!(store, addr, 1);

    s.write(Address::v(addr), op.rtv(s) as u16)?;

    Ok(None)
}

pub fn sh_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "SH {}, {:#06X}({})",
        op.rtn(),
        op.imm16(),
        op.rsn()
    ))
    //.with_address_hint(op.offset_addr(s))
}

fn sll_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.gpr[op.rd()].set(op.rtv(s) << op.shift());

    Ok(None)
}

pub fn sll_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(if op.rd() == 0 && op.rt() == 0 {
        "NOP".to_string()
    } else {
        format!("SLL {}, {}, {}", op.rdn(), op.rtn(), op.shift())
    })
}

fn sllv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.gpr[op.rd()].set(op.rtv(s) << (op.rsv(s) & 0x1F));

    Ok(None)
}

pub fn sllv_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("SLLV {}, {}, {}", op.rdn(), op.rtn(), op.rsn()))
}

fn slt_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.gpr[op.rd()].set64(((op.rsv64(s) as i64) < (op.rtv64(s) as i64)) as u64);

    Ok(None)
}

pub fn slt_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("SLT {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
}

fn slti_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.gpr[op.rt()].set64(((op.rsv64(s) as i64) < (op.imm16() as i16 as i64)) as u64);

    Ok(None)
}

pub fn slti_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "SLTI {}, {}, {:#06X}",
        op.rtn(),
        op.rsn(),
        op.imm16()
    ))
}

fn sltiu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let imm = op.imm16() as i16 as i64 as u64;

    s.cpu.regs.gpr[op.rt()].set64((op.rsv64(s) < imm) as u64);

    Ok(None)
}

pub fn sltiu_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "SLTIU {}, {}, {:#06X}",
        op.rtn(),
        op.rsn(),
        op.imm16()
    ))
}

fn sltu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.gpr[op.rd()].set64((op.rsv64(s) < op.rtv64(s)) as u64);

    Ok(None)
}

pub fn sltu_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("SLTU {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
}

fn sra_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let res = (op.rtv64(s) >> op.shift()) as i32 as i64 as u64;
    s.cpu.regs.gpr[op.rd()].set64(res);

    Ok(None)
}

pub fn sra_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("SRA {}, {}, {}", op.rdn(), op.rtn(), op.shift()))
}

fn srav_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let res = (op.rtv64(s) >> (op.rsv(s) & 0x1F)) as i32 as i64 as u64;
    s.cpu.regs.gpr[op.rd()].set64(res);

    Ok(None)
}

pub fn srav_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("SRAV {}, {}, {}", op.rdn(), op.rtn(), op.rsn()))
}

fn srl_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.gpr[op.rd()].set(op.rtv(s) >> op.shift());

    Ok(None)
}

pub fn srl_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("SRL {}, {}, {}", op.rdn(), op.rtn(), op.shift()))
}

fn srlv_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.gpr[op.rd()].set(op.rtv(s) >> (op.rsv(s) & 0x1F));

    Ok(None)
}

pub fn srlv_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("SRLV {}, {}, {}", op.rdn(), op.rtn(), op.rsn()))
}

fn sub_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let rs = op.rsv(s) as i32;
    let rt = op.rtv(s) as i32;
    match rs.checked_sub(rt) {
        Some(result) => {
            s.cpu.regs.gpr[op.rd()].set(result as u32);
            Ok(None)
        }
        None => Err(Exception::ArithmeticOverflow),
    }
}

pub fn sub_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("SUB {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
}

fn subu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.gpr[op.rd()].set(op.rsv(s).wrapping_sub(op.rtv(s)));

    Ok(None)
}

pub fn subu_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("SUBU {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
}

fn sw_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);
    check_aligned!(store, addr, 3);

    s.write(Address::v(addr), op.rtv(s))?;

    Ok(None)
}

pub fn sw_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "SW {}, {:#06X}({})",
        op.rtn(),
        op.imm16(),
        op.rsn()
    ))
    //.with_address_hint(op.offset_addr(s))
}

fn swc1_execute(s: &mut System, op: Opcode) -> InstructionResult {
    check_cop_usable!(1, s);

    let addr = op.offset_addr(s);
    check_aligned!(store, addr, 3);

    s.write(Address::v(addr), s.cop1.get32(op.ft(), s.cop0.f64()))?;

    Ok(None)
}

pub fn swc1_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "SWC1 {}, {:#06X}({})",
        op.ftn(),
        op.imm16(),
        op.basen()
    ))
}

fn sync_execute(_s: &mut System, _op: Opcode) -> InstructionResult {
    // TODO??

    Ok(None)
}

pub fn sync_disassemble(_s: &System, _op: Opcode) -> Disassembly {
    Disassembly::new("SYNC".to_string())
}

fn swl_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);
    let addr_base = addr & !3;
    let addr_offset = addr & 3;

    let word = if addr_offset == 0 {
        op.rtv(s)
    } else {
        let mut word = s.peek(Address::v(addr_base)).expect("Invalid address"); // TODO dangerous to read because of side effects?? (exceptions)
        word &= 0xFFFF_FFFF << (32 - 8 * addr_offset);
        word |= op.rtv(s) >> (8 * addr_offset);
        word
    };

    s.write(Address::v(addr_base), word)?;

    Ok(None)
}

pub fn swl_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "SWL {}, {:#06X}({})",
        op.rtn(),
        op.imm16(),
        op.rsn()
    ))
    //.with_address_hint(op.offset_addr(s))
}

fn swr_execute(s: &mut System, op: Opcode) -> InstructionResult {
    let addr = op.offset_addr(s);
    let base = addr & !3;
    let offset = addr & 3;

    let word = if offset == 3 {
        op.rtv(s)
    } else {
        let mut word = s.peek(Address::v(base)).expect("Invalid address"); // TODO dangerous to read because of side effects?? (exceptions)
        word &= 0xFFFF_FFFF >> (8 * (offset + 1));
        word |= op.rtv(s) << (24 - 8 * offset);
        word
    };

    s.write(Address::v(base), word)?;

    Ok(None)
}

pub fn swr_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "SWR {}, {:#06X}({})",
        op.rtn(),
        op.imm16(),
        op.rsn()
    ))
    //.with_address_hint(op.offset_addr(s))
}

fn syscall_execute(_s: &mut System, _op: Opcode) -> InstructionResult {
    Err(Exception::Syscall)
}

pub fn syscall_disassemble(_s: &System, _op: Opcode) -> Disassembly {
    Disassembly::new("SYSCALL".to_string())
}

// TODO traps: use generic helper

fn teq_execute(s: &mut System, op: Opcode) -> InstructionResult {
    if op.rsv64(s) == op.rtv64(s) {
        Err(Exception::Trap)
    } else {
        Ok(None)
    }
}

pub fn teq_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("TEQ {}, {}", op.rsn(), op.rtn()))
}

fn teqi_execute(s: &mut System, op: Opcode) -> InstructionResult {
    if (op.rsv64(s) as i64) == (op.imm16() as i16 as i64) {
        Err(Exception::Trap)
    } else {
        Ok(None)
    }
}

pub fn teqi_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("TEQI {}, {:#06X}", op.rsn(), op.imm16()))
}

fn tge_execute(s: &mut System, op: Opcode) -> InstructionResult {
    if (op.rsv64(s) as i64) >= (op.rtv64(s) as i64) {
        Err(Exception::Trap)
    } else {
        Ok(None)
    }
}

pub fn tge_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("TGE {}, {}", op.rsn(), op.rtn()))
}

fn tgei_execute(s: &mut System, op: Opcode) -> InstructionResult {
    if (op.rsv64(s) as i64) >= (op.imm16() as i16 as i64) {
        Err(Exception::Trap)
    } else {
        Ok(None)
    }
}

pub fn tgei_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("TGEI {}, {:#06X}", op.rsn(), op.imm16()))
}

fn tgeiu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    if op.rsv64(s) >= op.imm16() as i16 as i64 as u64 {
        Err(Exception::Trap)
    } else {
        Ok(None)
    }
}

pub fn tgeiu_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("TGEIU {}, {:#06X}", op.rsn(), op.imm16()))
}

fn tgeu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    if op.rsv64(s) >= op.rtv64(s) {
        Err(Exception::Trap)
    } else {
        Ok(None)
    }
}

pub fn tgeu_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("TGEU {}, {}", op.rsn(), op.rtn()))
}

fn tlt_execute(s: &mut System, op: Opcode) -> InstructionResult {
    if (op.rsv64(s) as i64) < (op.rtv64(s) as i64) {
        Err(Exception::Trap)
    } else {
        Ok(None)
    }
}

pub fn tlt_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("TLT {}, {}", op.rsn(), op.rtn()))
}

fn tlti_execute(s: &mut System, op: Opcode) -> InstructionResult {
    if (op.rsv64(s) as i64) < (op.imm16() as i16 as i64) {
        Err(Exception::Trap)
    } else {
        Ok(None)
    }
}

pub fn tlti_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("TLTI {}, {:#06X}", op.rsn(), op.imm16()))
}

fn tltiu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    if op.rsv64(s) < op.imm16() as i16 as i64 as u64 {
        Err(Exception::Trap)
    } else {
        Ok(None)
    }
}

pub fn tltiu_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("TLTIU {}, {:#06X}", op.rsn(), op.imm16()))
}

fn tltu_execute(s: &mut System, op: Opcode) -> InstructionResult {
    if op.rsv64(s) < op.rtv64(s) {
        Err(Exception::Trap)
    } else {
        Ok(None)
    }
}

pub fn tltu_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("TLTU {}, {}", op.rsn(), op.rtn()))
}

fn tne_execute(s: &mut System, op: Opcode) -> InstructionResult {
    if op.rsv64(s) != op.rtv64(s) {
        Err(Exception::Trap)
    } else {
        Ok(None)
    }
}

pub fn tne_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("TNE {}, {}", op.rsn(), op.rtn()))
}

fn tnei_execute(s: &mut System, op: Opcode) -> InstructionResult {
    if (op.rsv64(s) as i64) != (op.imm16() as i16 as i64) {
        Err(Exception::Trap)
    } else {
        Ok(None)
    }
}

pub fn tnei_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("TNEI {}, {:#06X}", op.rsn(), op.imm16()))
}

fn xor_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.gpr[op.rd()].set64(op.rsv64(s) ^ op.rtv64(s));

    Ok(None)
}

pub fn xor_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("XOR {}, {}, {}", op.rdn(), op.rsn(), op.rtn()))
}

fn xori_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.gpr[op.rt()].set64(op.rsv64(s) ^ op.imm16() as u64);

    Ok(None)
}

pub fn xori_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "XORI {}, {}, {:#06X}",
        op.rtn(),
        op.rsn(),
        op.imm16()
    ))
}
