#![allow(clippy::upper_case_acronyms)]

use super::{Disassembly, Instruction, InstructionResult, Opcode, System};
use crate::{
    exception::Exception, instruction_struct, instructions::UNKNOWN_, registers::Registers,
};

// TODO use const generics to share code with cop1?

/// COP2 rs field (bits 25–21).
fn cop2_rs(opcode: Opcode) -> u32 {
    (opcode.0 >> 21) & 0x1F
}

pub fn decode(opcode: Opcode) -> Option<&'static dyn Instruction> {
    debug_assert_eq!(opcode.group(), 0x12);

    let instruction: &'static dyn Instruction = match cop2_rs(opcode) {
        0x00 => &MFC2_,
        0x01 => &DMFC2_,
        0x02 => &CFC2_,
        //0x03 => &DCFC1_,
        0x04 => &MTC2_,
        0x05 => &DMTC2_,
        0x06 => &CTC2_,
        //0x07 => &DCTC1_,
        _ => &UNKNOWN_,
    };

    Some(instruction)
}

instruction_struct!(CFC2);

impl Instruction for CFC2 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        assert!(op.fs() == 31); // TODO 0 too?

        if !s.cop0.cop2_usable() {
            return Some(InstructionResult::Exception(
                Exception::CoprocessorUnusable(2),
            ));
        }

        s.cpu.regs.gpr[op.rt()].set(op.fsv(s));

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "CFC2 {}, {}",
            op.rtn(),
            Registers::fpr_name(op.rd())
        ))
    }
}

instruction_struct!(CTC2);

impl Instruction for CTC2 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        assert!(op.fs() == 31); // TODO 0 too?

        if !s.cop0.cop2_usable() {
            return Some(InstructionResult::Exception(
                Exception::CoprocessorUnusable(2),
            ));
        }

        s.cpu.regs.fcr = op.fsv(s);

        // TODO exceptions

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        // TODO
        Disassembly::new(format!(
            "CTC2 {}, {}",
            op.rtn(),
            Registers::fpr_name(op.rd())
        ))
    }
}

instruction_struct!(DMFC2);

impl Instruction for DMFC2 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        if !s.cop0.cop2_usable() {
            return Some(InstructionResult::Exception(
                Exception::CoprocessorUnusable(2),
            ));
        }

        let freg = op.fs();

        if s.cop0.f_64() {
            s.cpu.regs.gpr[op.rt()].set64(s.cpu.regs.fpr[freg].get64());
        } else {
            s.cpu.regs.gpr[op.rt()].set64(s.cpu.regs.fpr[freg & !1].get64());
        }

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DMFC2 {}, {}", op.rtn(), op.fsn()))
    }
}

instruction_struct!(DMTC2);

impl Instruction for DMTC2 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        if !s.cop0.cop2_usable() {
            return Some(InstructionResult::Exception(
                Exception::CoprocessorUnusable(2),
            ));
        }

        let freg = op.fs();

        if s.cop0.f_64() {
            s.cpu.regs.fpr[freg].set64(op.rtv64(s));
        } else {
            s.cpu.regs.fpr[freg & !1].set64(op.rtv64(s));
        }

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DMTC2 {}, {}", op.rtn(), op.rd0n()))
    }
}

instruction_struct!(MFC2);

impl Instruction for MFC2 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        if !s.cop0.cop2_usable() {
            return Some(InstructionResult::Exception(
                Exception::CoprocessorUnusable(2),
            ));
        }

        let value = if s.cop0.f_64() || op.fs() & 1 == 0 {
            op.fsv(s)
        } else {
            (s.cpu.regs.fpr[op.fs() & !1].get64() >> 32) as u32
        };

        s.cpu.regs.gpr[op.rt()].set(value);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("MFC2 {}, {}", op.rtn(), op.rd0n())) // TODO FPreg!
    }
}

instruction_struct!(MTC2);

impl Instruction for MTC2 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        if !s.cop0.cop2_usable() {
            return Some(InstructionResult::Exception(
                Exception::CoprocessorUnusable(2),
            ));
        }

        let freg = op.fs();
        let fval = s.cpu.regs.fpr[freg].get64();

        if s.cop0.f_64() || freg & 1 == 0 {
            s.cpu.regs.fpr[freg].set64((fval & 0xFFFFFFFF_00000000) | (op.rtv(s) as u64));
        } else {
            s.cpu.regs.fpr[freg & !1]
                .set64((fval & 0x00000000_FFFFFFFF) | ((op.rtv(s) as u64) << 32));
        }

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("MTC2 {}, {}", op.rtn(), op.rd0n()))
    }
}
