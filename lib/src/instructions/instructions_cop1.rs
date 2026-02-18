use super::{DelayedBranching, Disassembly, Instruction, Opcode, System};
use crate::{instruction_struct, instructions::UNKNOWN_, registers::Registers};

pub fn decode(opcode: Opcode) -> Option<&'static dyn Instruction> {
    debug_assert_eq!(opcode.group(), 0x11);

    let instruction: &'static dyn Instruction = match opcode.0 & 0x3E0_0000 {
        0x00_0000 => &MFC1_,
        0x40_0000 => &CFC1_,
        0x80_0000 => &MTC1_,
        0xC0_0000 => &CTC1_,
        _ => &UNKNOWN_,
    };

    if std::ptr::eq(instruction, &UNKNOWN_) {
        None
    } else {
        Some(instruction)
    }
}

instruction_struct!(CFC1);

impl Instruction for CFC1 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        s.cpu.regs.gpr[op.rt()].set64(s.cpu.regs.fpr[op.rd()] as u64);
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "CFC1 {}, {}",
            op.rtn(),
            Registers::fpr_name(op.rd())
        ))
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
        Disassembly::new(format!(
            "CTC1 {}, {}",
            op.rtn(),
            Registers::fpr_name(op.rd())
        ))
    }
}

instruction_struct!(MFC1);

impl Instruction for MFC1 {
    fn execute(&self, _s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        panic!("MFC1 {}, {}", op.rtn(), op.rd0n());

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("MFC1 {}, {}", op.rtn(), op.rd0n())) // TODO FPreg!
    }
}

instruction_struct!(MTC1);

impl Instruction for MTC1 {
    fn execute(&self, _s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        // TODO

        log::warn!("MTC1 {}, {}", op.rtn(), op.rd0n());

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("MTC1 {}, {}", op.rtn(), op.rd0n()))
    }
}
