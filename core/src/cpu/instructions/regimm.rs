use crate::{
    cpu::{
        instructions::{Instruction, InstructionResult, branch, trap},
        opcode::Opcode,
        operands::Operands,
    },
    exception::Exception,
    system::System,
};

#[macro_export]
macro_rules! decode_regimm_x {
    ($opcode:expr, $m:ident) => {{
        debug_assert_eq!($opcode.group(), 0x01);

        match $opcode.0 & 0x1F_0000 {
            0x00_0000 => $m!(crate::cpu::instructions::regimm::Bltz),
            0x01_0000 => $m!(crate::cpu::instructions::regimm::Bgez),
            0x02_0000 => $m!(crate::cpu::instructions::regimm::Bltzl),
            0x03_0000 => $m!(crate::cpu::instructions::regimm::Bgezl),
            0x08_0000 => $m!(crate::cpu::instructions::regimm::Tgei),
            0x09_0000 => $m!(crate::cpu::instructions::regimm::Tgeiu),
            0x0A_0000 => $m!(crate::cpu::instructions::regimm::Tlti),
            0x0B_0000 => $m!(crate::cpu::instructions::regimm::Tltiu),
            0x0C_0000 => $m!(crate::cpu::instructions::regimm::Teqi),
            0x0E_0000 => $m!(crate::cpu::instructions::regimm::Tnei),
            0x10_0000 => $m!(crate::cpu::instructions::regimm::Bltzal),
            0x11_0000 => $m!(crate::cpu::instructions::regimm::Bgezal),
            0x13_0000 => $m!(crate::cpu::instructions::regimm::Bgezall),
            _ => $m!(crate::cpu::instructions::Reserved),
        }
    }};
}

// ---------
// Branching
// ---------

pub struct Bltz;

impl Instruction for Bltz {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        branch::<false>(s, opcode, (operands.rsv64(s) as i64) < 0)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!("BLTZ {}, {:#06X}", operands.rsn(), opcode.branch_offset())
    }
}

pub struct Bgez;

impl Instruction for Bgez {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        branch::<false>(s, opcode, (operands.rsv64(s) as i64) >= 0)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!("BGEZ {}, {:#06X}", operands.rsn(), opcode.branch_offset())
    }
}

pub struct Bltzl;

impl Instruction for Bltzl {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        branch::<true>(s, opcode, (operands.rsv64(s) as i64) < 0)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!("BLTZL {}, {:#06X}", operands.rsn(), opcode.branch_offset())
    }
}

pub struct Bgezl;

impl Instruction for Bgezl {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        branch::<true>(s, opcode, (operands.rsv64(s) as i64) >= 0)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!("BGEZL {}, {:#06X}", operands.rsn(), opcode.branch_offset())
    }
}

pub struct Bltzal;

impl Instruction for Bltzal {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        // Read before linking (matters when rs == 31)
        let rs = operands.rsv64(s) as i64;

        // The return address is the instruction that follows the delay slot
        s.cpu.regs.gpr[31].set(s.cpu.regs.pc.wrapping_add(8));

        branch::<false>(s, opcode, rs < 0)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!("BLTZAL {}, {:#06X}", operands.rsn(), opcode.branch_offset())
    }
}

pub struct Bgezal;

impl Instruction for Bgezal {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        // Read before linking (matters when rs == 31)
        let rs = operands.rsv64(s) as i64;

        // The return address is the instruction that follows the delay slot
        s.cpu.regs.gpr[31].set(s.cpu.regs.pc.wrapping_add(8));

        branch::<false>(s, opcode, rs >= 0)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!("BGEZAL {}, {:#06X}", operands.rsn(), opcode.branch_offset())
    }
}

pub struct Bgezall;

impl Instruction for Bgezall {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        // Read before linking (matters when rs == 31)
        let rs = operands.rsv64(s) as i64;

        // The return address is the instruction that follows the delay slot
        s.cpu.regs.gpr[31].set(s.cpu.regs.pc.wrapping_add(8));

        branch::<true>(s, opcode, rs as i64 >= 0)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!(
            "BGEZALL {}, {:#06X}",
            operands.rsn(),
            opcode.branch_offset()
        )
        // TODO cond result?
    }
}

// -----
// Traps
// -----

pub struct Tgei;

impl Instruction for Tgei {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        trap((operands.rsv64(s) as i64) >= (opcode.imm16() as i16 as i64))
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!("TGEI {}, {:#06X}", operands.rsn(), opcode.imm16())
    }
}

pub struct Tgeiu;

impl Instruction for Tgeiu {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        trap(operands.rsv64(s) >= opcode.imm16() as i16 as i64 as u64)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!("TGEIU {}, {:#06X}", operands.rsn(), opcode.imm16())
    }
}

pub struct Tlti;

impl Instruction for Tlti {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        trap((operands.rsv64(s) as i64) < (opcode.imm16() as i16 as i64))
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!("TLTI {}, {:#06X}", operands.rsn(), opcode.imm16())
    }
}

pub struct Tltiu;

impl Instruction for Tltiu {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        trap(operands.rsv64(s) < opcode.imm16() as i16 as i64 as u64)
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!("TLTIU {}, {:#06X}", operands.rsn(), opcode.imm16())
    }
}

pub struct Teqi;

impl Instruction for Teqi {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        trap((operands.rsv64(s) as i64) == (opcode.imm16() as i16 as i64))
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!("TEQI {}, {:#06X}", operands.rsn(), opcode.imm16())
    }
}

pub struct Tnei;

impl Instruction for Tnei {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult {
        if (operands.rsv64(s) as i64) != (opcode.imm16() as i16 as i64) {
            Err(Exception::Trap)
        } else {
            Ok(None)
        }
    }

    fn disassemble(_s: &System, opcode: Opcode, operands: Operands) -> String {
        format!("TNEI {}, {:#06X}", operands.rsn(), opcode.imm16())
    }
}
