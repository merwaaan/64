use super::{DisassembleFn, Disassembly, ExecuteFn, InstructionResult, Opcode, System};
use crate::{exception::Exception, inst, registers::Registers};

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

fn cfc2_execute(s: &mut System, op: Opcode) -> Option<InstructionResult> {
    assert!(op.fs() == 31);

    if !s.cop0.cop2_usable() {
        return Some(InstructionResult::Exception(
            Exception::CoprocessorUnusable(2),
        ));
    }

    s.cpu.regs.gpr[op.rt()].set(op.fsv(s));

    None
}

fn cfc2_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "CFC2 {}, {}",
        op.rtn(),
        Registers::fpr_name(op.rd())
    ))
}

fn ctc2_execute(s: &mut System, _op: Opcode) -> Option<InstructionResult> {
    if !s.cop0.cop2_usable() {
        return Some(InstructionResult::Exception(
            Exception::CoprocessorUnusable(2),
        ));
    }

    log::error!("UNIMPLEMENTED CTC2");

    None
}

fn ctc2_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!(
        "CTC2 {}, {}",
        op.rtn(),
        Registers::fpr_name(op.rd())
    ))
}

fn dmfc2_execute(s: &mut System, _op: Opcode) -> Option<InstructionResult> {
    if !s.cop0.cop2_usable() {
        return Some(InstructionResult::Exception(
            Exception::CoprocessorUnusable(2),
        ));
    }

    log::error!("UNIMPLEMENTED DMFC2");

    None
}

fn dmfc2_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DMFC2 {}, {}", op.rtn(), op.fsn()))
}

fn dmtc2_execute(s: &mut System, _op: Opcode) -> Option<InstructionResult> {
    if !s.cop0.cop2_usable() {
        return Some(InstructionResult::Exception(
            Exception::CoprocessorUnusable(2),
        ));
    }

    log::error!("UNIMPLEMENTED DMTC2");

    None
}

fn dmtc2_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DMTC2 {}, {}", op.rtn(), op.rd0n()))
}

fn mfc2_execute(s: &mut System, _op: Opcode) -> Option<InstructionResult> {
    if !s.cop0.cop2_usable() {
        return Some(InstructionResult::Exception(
            Exception::CoprocessorUnusable(2),
        ));
    }

    log::error!("UNIMPLEMENTED MFC2");

    None
}

fn mfc2_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("MFC2 {}, {}", op.rtn(), op.rd0n()))
}

fn mtc2_execute(s: &mut System, _op: Opcode) -> Option<InstructionResult> {
    if !s.cop0.cop2_usable() {
        return Some(InstructionResult::Exception(
            Exception::CoprocessorUnusable(2),
        ));
    }

    log::error!("UNIMPLEMENTED MTC2");

    None
}

fn mtc2_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("MTC2 {}, {}", op.rtn(), op.rd0n()))
}
