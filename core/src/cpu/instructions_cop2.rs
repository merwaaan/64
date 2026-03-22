use crate::{
    check_cop_usable,
    cpu::{
        instructions::{DisassembleFn, Disassembly, ExecuteFn, InstructionResult},
        opcode::Opcode,
    },
    exception::Exception,
    inst,
    registers::Registers,
    system::System,
};

fn cop2_rs(opcode: Opcode) -> u32 {
    (opcode.0 >> 21) & 0x1F
}

pub fn decode(opcode: Opcode) -> Option<(ExecuteFn, DisassembleFn)> {
    debug_assert_eq!(opcode.group(), 0x12);

    Some(match cop2_rs(opcode) {
        0x00 => inst!(mfc2),
        0x01 => inst!(dmfc2),
        0x02 => inst!(cfc2),
        0x04 => inst!(mtc2),
        0x05 => inst!(dmtc2),
        0x06 => inst!(ctc2),
        _ => return None,
    })
}

fn cfc2_execute(s: &mut System, _op: Opcode) -> InstructionResult {
    check_cop_usable!(2, s);

    log::warn!("UNIMPLEMENTED CFC2");
    // s.cpu.regs.gpr[op.rt()].set(op.fsv(s));

    Ok(None)
}

fn cfc2_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "CFC2 {}, {}",
        op.rtn(),
        Registers::fpr_name(op.rd())
    ))
}

fn ctc2_execute(s: &mut System, _op: Opcode) -> InstructionResult {
    check_cop_usable!(2, s);

    log::warn!("UNIMPLEMENTED CTC2");

    Ok(None)
}

fn ctc2_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "CTC2 {}, {}",
        op.rtn(),
        Registers::fpr_name(op.rd())
    ))
}

fn dmfc2_execute(s: &mut System, _op: Opcode) -> InstructionResult {
    check_cop_usable!(2, s);

    log::warn!("UNIMPLEMENTED DMFC2");

    Ok(None)
}

fn dmfc2_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DMFC2 {}, {}", op.rtn(), op.fsn()))
}

fn dmtc2_execute(s: &mut System, _op: Opcode) -> InstructionResult {
    check_cop_usable!(2, s);

    log::warn!("UNIMPLEMENTED DMTC2");

    Ok(None)
}

fn dmtc2_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DMTC2 {}, {}", op.rtn(), op.rd0n()))
}

fn mfc2_execute(s: &mut System, _op: Opcode) -> InstructionResult {
    check_cop_usable!(2, s);

    log::warn!("UNIMPLEMENTED MFC2");

    Ok(None)
}

fn mfc2_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("MFC2 {}, {}", op.rtn(), op.rd0n()))
}

fn mtc2_execute(s: &mut System, _op: Opcode) -> InstructionResult {
    check_cop_usable!(2, s);

    log::warn!("UNIMPLEMENTED MTC2");

    Ok(None)
}

fn mtc2_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("MTC2 {}, {}", op.rtn(), op.rd0n()))
}
