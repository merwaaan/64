use super::{DecodedInstruction, Disassembly, InstructionResult, Opcode, System};
use crate::inst;

pub fn decode(opcode: Opcode) -> Option<DecodedInstruction> {
    debug_assert_eq!(opcode.group(), 0x10);

    Some(match opcode.0 & 0x03E0_0000 {
        0x000_0000 => inst!(mfc0),
        0x020_0000 => inst!(dmfc0),
        0x080_0000 => inst!(mtc0),
        0x0A0_0000 => inst!(dmtc0),
        0x200_0000 => match opcode.0 & 0x3F {
            0x01 => inst!(tlbr),
            0x02 => inst!(tlbwi),
            0x08 => inst!(tlbp),
            0x18 => inst!(eret),
            _ => return None,
        },
        _ => return None,
    })
}

fn dmfc0_execute(s: &mut System, op: Opcode) -> Option<InstructionResult> {
    s.cpu.regs.gpr[op.rt()].set64(s.cop0.read(op.rd()).get64());

    None
}

fn dmfc0_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DMFC0 {}, {}", op.rtn(), op.rd0n()))
}

fn dmtc0_execute(s: &mut System, op: Opcode) -> Option<InstructionResult> {
    s.cop0.write64(op.rd(), op.rtv64(s));

    None
}

fn dmtc0_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("DMTC0 {}, {}", op.rtn(), op.rd0n()))
}

fn eret_execute(s: &mut System, _op: Opcode) -> Option<InstructionResult> {
    if s.cop0.erl() {
        unimplemented!("ERET in ERL mode");
    } else {
        s.cpu.regs.pc = s.cop0.epc().wrapping_sub(4);
        s.cop0.clear_exl();
    }

    s.cpu.regs.load_linked_bit = false;

    None
}

fn eret_disassemble(_s: &System, _op: Opcode) -> Disassembly {
    Disassembly::new("ERET".to_string())
}

fn mtc0_execute(s: &mut System, op: Opcode) -> Option<InstructionResult> {
    s.cop0.write(op.rd(), op.rtv(s));

    None
}

fn mtc0_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("MTC0 {}, {}", op.rtn(), op.rd0n()))
}

fn mfc0_execute(s: &mut System, op: Opcode) -> Option<InstructionResult> {
    s.cpu.regs.gpr[op.rt()].set(s.cop0.read(op.rd()).get());

    None
}

fn mfc0_disassemble(_s: &System, op: Opcode) -> Disassembly {
    Disassembly::new(format!("MFC0 {}, {}", op.rtn(), op.rd0n()))
}

fn tlbp_execute(s: &mut System, _op: Opcode) -> Option<InstructionResult> {
    log::warn!("TLBP @ {:08X}", s.cpu.regs.pc);

    None
}

fn tlbp_disassemble(_s: &System, _op: Opcode) -> Disassembly {
    Disassembly::new("TLBP".to_string())
}

fn tlbr_execute(s: &mut System, _op: Opcode) -> Option<InstructionResult> {
    log::warn!(
        "TLBR @ {:08X} (index={})",
        s.cpu.regs.pc,
        s.cop0.read(0).get()
    );

    None
}

fn tlbr_disassemble(_s: &System, _op: Opcode) -> Disassembly {
    Disassembly::new("TLBR".to_string())
}

fn tlbwi_execute(s: &mut System, _op: Opcode) -> Option<InstructionResult> {
    log::warn!(
        "TLBWI @ {:08X} (index={})",
        s.cpu.regs.pc,
        s.cop0.read(0).get()
    );

    None
}

fn tlbwi_disassemble(_s: &System, _op: Opcode) -> Disassembly {
    Disassembly::new("TLBWI".to_string())
}
