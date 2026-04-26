use crate::{
    check_cop_usable,
    cpu::{
        instructions::{Instruction, InstructionResult},
        opcode::Opcode,
        operands::Operands,
    },
    exception::Exception,
    system::System,
};

#[macro_export]
macro_rules! decode_cop2_x {
    ($opcode:expr, $m:ident) => {{
        match $opcode.rs() {
            0x00 => $m!(crate::cpu::instructions::cop2::Mfc2),
            0x01 => $m!(crate::cpu::instructions::cop2::Dmfc2),
            0x02 => $m!(crate::cpu::instructions::cop2::Cfc2),
            0x04 => $m!(crate::cpu::instructions::cop2::Mtc2),
            0x05 => $m!(crate::cpu::instructions::cop2::Dmtc2),
            0x06 => $m!(crate::cpu::instructions::cop2::Ctc2),
            _ => $m!(crate::cpu::instructions::Reserved),
        }
    }};
}

// --------------------------
// Move to/from coprocessor 2
// --------------------------

pub struct Cfc2;

impl Instruction for Cfc2 {
    fn execute(s: &mut System, _opcode: Opcode, _operands: Operands) -> InstructionResult {
        check_cop_usable!(2, s);

        log::warn!("UNIMPLEMENTED CFC2");
        // s.cpu.regs.gpr[op.rt()].set(op.fsv(s));

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("CFC2 {}, {}", operands.rtn(), operands.fdn())
    }
}

pub struct Ctc2;

impl Instruction for Ctc2 {
    fn execute(s: &mut System, _opcode: Opcode, _operands: Operands) -> InstructionResult {
        check_cop_usable!(2, s);

        log::warn!("UNIMPLEMENTED CTC2");

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("CTC2 {}, {}", operands.rtn(), operands.fdn())
    }
}
pub struct Dmfc2;

impl Instruction for Dmfc2 {
    fn execute(s: &mut System, _opcode: Opcode, _operands: Operands) -> InstructionResult {
        check_cop_usable!(2, s);

        log::warn!("UNIMPLEMENTED DMFC2");

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("DMFC2 {}, {}", operands.rtn(), operands.fsn())
    }
}

pub struct Dmtc2;

impl Instruction for Dmtc2 {
    fn execute(s: &mut System, _opcode: Opcode, _operands: Operands) -> InstructionResult {
        check_cop_usable!(2, s);

        log::warn!("UNIMPLEMENTED DMTC2");

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("DMTC2 {}, {}", operands.rtn(), operands.rd0n())
    }
}

pub struct Mfc2;

impl Instruction for Mfc2 {
    fn execute(s: &mut System, _opcode: Opcode, _operands: Operands) -> InstructionResult {
        check_cop_usable!(2, s);

        log::warn!("UNIMPLEMENTED MFC2");

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("MFC2 {}, {}", operands.rtn(), operands.rd0n())
    }
}

pub struct Mtc2;

impl Instruction for Mtc2 {
    fn execute(s: &mut System, _opcode: Opcode, _operands: Operands) -> InstructionResult {
        check_cop_usable!(2, s);

        log::warn!("UNIMPLEMENTED MTC2");

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("MTC2 {}, {}", operands.rtn(), operands.rd0n())
    }
}
