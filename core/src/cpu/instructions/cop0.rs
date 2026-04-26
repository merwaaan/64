use crate::{
    cop0,
    cpu::{
        instructions::{Instruction, InstructionResult},
        opcode::Opcode,
        operands::Operands,
    },
    system::System,
    tlb,
};

#[macro_export]
macro_rules! decode_cop0_x {
    ($opcode:expr, $m:ident) => {{
        debug_assert_eq!($opcode.group(), 0x10);

        // TODO mips manual shows a complex table (p. 544), not sure if it's worth implementing

        match $opcode.0 & 0x03E0_0000 {
            0x000_0000 => $m!(crate::cpu::instructions::cop0::Mfc0),
            0x020_0000 => $m!(crate::cpu::instructions::cop0::Dmfc0),
            0x080_0000 => $m!(crate::cpu::instructions::cop0::Mtc0),
            0x0A0_0000 => $m!(crate::cpu::instructions::cop0::Dmtc0),
            0x200_0000 => match $opcode.0 & 0x3F {
                0x01 => $m!(crate::cpu::instructions::cop0::Tlbr),
                0x02 => $m!(crate::cpu::instructions::cop0::Tlbwi),
                0x04 => $m!(crate::cpu::instructions::cop0::Tlbwr),
                0x08 => $m!(crate::cpu::instructions::cop0::Tlbp),
                0x18 => $m!(crate::cpu::instructions::cop0::Eret),
                _ => $m!(crate::cpu::instructions::Reserved),
            },
            _ => $m!(crate::cpu::instructions::Reserved),
        }
    }};
}

pub struct Eret;

impl Instruction for Eret {
    fn execute(s: &mut System, _opcode: Opcode, _operands: Operands) -> InstructionResult {
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

    fn disassemble(_s: &System, _opcode: Opcode, _operands: Operands) -> String {
        "ERET".to_string()
    }
}

// --------------------------
// Move to/from coprocessor 0
// --------------------------

pub struct Mfc0;

impl Instruction for Mfc0 {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.gpr[operands.rt()].set(s.cop0.read(operands.rd()).get());

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("MFC0 {}, {}", operands.rtn(), operands.rd0n())
    }
}

pub struct Dmfc0;

impl Instruction for Dmfc0 {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cpu.regs.gpr[operands.rt()].set64(s.cop0.read(operands.rd()).get64());

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("DMFC0 {}, {}", operands.rtn(), operands.rd0n())
    }
}

pub struct Mtc0;

impl Instruction for Mtc0 {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cop0.write(operands.rd(), operands.rtv(s));

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("MTC0 {}, {}", operands.rtn(), operands.rd0n())
    }
}

pub struct Dmtc0;

impl Instruction for Dmtc0 {
    fn execute(s: &mut System, _opcode: Opcode, operands: Operands) -> InstructionResult {
        s.cop0.write64(operands.rd(), operands.rtv64(s));

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, operands: Operands) -> String {
        format!("DMTC0 {}, {}", operands.rtn(), operands.rd0n())
    }
}

// ---
// TLB
// ---

pub struct Tlbp;

impl Instruction for Tlbp {
    fn execute(s: &mut System, _opcode: Opcode, _operands: Operands) -> InstructionResult {
        if let Some(index) = s.cop0.tlb.probe(&s.cop0) {
            s.cop0.write(cop0::Register::Index as usize, index as u32);
        } else {
            s.cop0.write(cop0::Register::Index as usize, 0x8000_0000);
        }

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, _operands: Operands) -> String {
        "TLBP".to_string()
    }
}

pub struct Tlbr;

impl Instruction for Tlbr {
    fn execute(s: &mut System, _opcode: Opcode, _operands: Operands) -> InstructionResult {
        s.cop0
            .tlb
            .read(s.cop0.read(cop0::Register::Index as usize).get())
            .to_cop0_regs(&mut s.cop0);

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, _operands: Operands) -> String {
        "TLBR".to_string()
    }
}

pub struct Tlbwi;

impl Instruction for Tlbwi {
    fn execute(s: &mut System, _opcode: Opcode, _operands: Operands) -> InstructionResult {
        s.cop0.tlb.write(
            s.cop0.read(cop0::Register::Index as usize).get(),
            tlb::Entry::from_cop0_regs(&s.cop0),
        );

        Ok(None)
    }

    fn disassemble(_s: &System, _opcode: Opcode, _operands: Operands) -> String {
        "TLBWI".to_string()
    }
}

pub struct Tlbwr;

impl Instruction for Tlbwr {
    fn execute(s: &mut System, _opcode: Opcode, _operands: Operands) -> InstructionResult {
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

    fn disassemble(_s: &System, _opcode: Opcode, _operands: Operands) -> String {
        "TLBWR".to_string()
    }
}
