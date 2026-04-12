use crate::{
    cop0,
    cpu::{
        instructions::{DecodedInstruction, InstructionResult, RESERVED_INSTRUCTION},
        opcode::Opcode,
    },
    inst,
    system::System,
    tlb,
};

pub fn decode(opcode: Opcode) -> DecodedInstruction {
    debug_assert_eq!(opcode.group(), 0x10);

    match opcode.0 & 0x03E0_0000 {
        0x000_0000 => inst!(mfc0),
        0x020_0000 => inst!(dmfc0),
        0x080_0000 => inst!(mtc0),
        0x0A0_0000 => inst!(dmtc0),
        0x200_0000 => match opcode.0 & 0x3F {
            0x01 => inst!(tlbr),
            0x02 => inst!(tlbwi),
            0x04 => inst!(tlbwr),
            0x08 => inst!(tlbp),
            0x18 => inst!(eret),
            _ => RESERVED_INSTRUCTION,
        },
        _ => RESERVED_INSTRUCTION,
    }
}

fn dmfc0_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.gpr[op.rt()].set64(s.cop0.read(op.rd()).get64());

    Ok(None)
}

fn dmfc0_disassemble(_s: &System, op: Opcode) -> String {
    format!("DMFC0 {}, {}", op.rtn(), op.rd0n())
}

fn dmtc0_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cop0.write64(op.rd(), op.rtv64(s));

    Ok(None)
}

fn dmtc0_disassemble(_s: &System, op: Opcode) -> String {
    format!("DMTC0 {}, {}", op.rtn(), op.rd0n())
}

fn eret_execute(s: &mut System, _op: Opcode) -> InstructionResult {
    if s.cop0.erl() {
        s.cpu.regs.pc = s.cop0.error_pc().wrapping_sub(4); // TODO why wrapping sub, hack?
        s.cop0.clear_erl();
    } else {
        s.cpu.regs.pc = s.cop0.exception_pc().wrapping_sub(4); // TODO why wrapping sub, hack?
        s.cop0.clear_exl();
    }

    s.cpu.regs.load_linked_bit = false;

    Ok(None)
}

fn eret_disassemble(_s: &System, _op: Opcode) -> String {
    "ERET".to_string()
}
fn mfc0_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cpu.regs.gpr[op.rt()].set(s.cop0.read(op.rd()).get());

    Ok(None)
}

fn mfc0_disassemble(_s: &System, op: Opcode) -> String {
    format!("MFC0 {}, {}", op.rtn(), op.rd0n())
}

fn mtc0_execute(s: &mut System, op: Opcode) -> InstructionResult {
    s.cop0.write(op.rd(), op.rtv(s));

    Ok(None)
}

fn mtc0_disassemble(_s: &System, op: Opcode) -> String {
    format!("MTC0 {}, {}", op.rtn(), op.rd0n())
}

fn tlbp_execute(s: &mut System, _op: Opcode) -> InstructionResult {
    if let Some(index) = s.cop0.tlb.probe(&s.cop0) {
        s.cop0.write(cop0::Register::Index as usize, index as u32);
    } else {
        s.cop0.write(cop0::Register::Index as usize, 0x8000_0000);
    }

    Ok(None)
}

fn tlbp_disassemble(_s: &System, _op: Opcode) -> String {
    "TLBP".to_string()
}

fn tlbr_execute(s: &mut System, _op: Opcode) -> InstructionResult {
    s.cop0
        .tlb
        .read(s.cop0.read(cop0::Register::Index as usize).get())
        .to_cop0_regs(&mut s.cop0);

    Ok(None)
}

fn tlbr_disassemble(_s: &System, _op: Opcode) -> String {
    "TLBR".to_string()
}

fn tlbwi_execute(s: &mut System, _op: Opcode) -> InstructionResult {
    s.cop0.tlb.write(
        s.cop0.read(cop0::Register::Index as usize).get(),
        tlb::Entry::from_cop0_regs(&s.cop0),
    );

    Ok(None)
}

fn tlbwi_disassemble(_s: &System, _op: Opcode) -> String {
    "TLBWI".to_string()
}

fn tlbwr_execute(s: &mut System, _op: Opcode) -> InstructionResult {
    log::warn!(
        "TLBWR @ {:08X} (index={})",
        s.cpu.regs.pc,
        s.cop0.read(0).get()
    );

    // TODO update random!

    s.cop0.tlb.write(
        s.cop0.read(cop0::Register::Random as usize).get(),
        tlb::Entry::from_cop0_regs(&s.cop0),
    );

    Ok(None)
}

fn tlbwr_disassemble(_s: &System, _op: Opcode) -> String {
    "TLBWR".to_string()
}
