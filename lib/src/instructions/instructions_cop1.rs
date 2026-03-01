#![allow(clippy::upper_case_acronyms)]

use super::{Disassembly, Instruction, InstructionResult, Opcode, System};
use crate::{
    cop1::{self, Format},
    exception::Exception,
    instruction_struct,
    instructions::UNKNOWN_,
    registers::Registers,
};

pub fn decode(opcode: Opcode) -> Option<&'static dyn Instruction> {
    debug_assert_eq!(opcode.group(), 0x11);

    // TODO can avoid & 1F as they all have the same prefix

    let instruction: &'static dyn Instruction = match (opcode.0 >> 21) & 0x1F {
        0x00 => &MFC1_,
        0x01 => &DMFC1_,
        0x02 => &CFC1_,
        0x04 => &MTC1_,
        0x05 => &DMTC1_,
        0x06 => &CTC1_,
        _ => match opcode.0 & 0x3F {
            0x00 => &ADD_,
            0x01 => &SUB_,
            0x05 => &ABS_,
            0x06 => &MOV_,
            0x07 => &NEG_,
            0x08 => &ROUND_,
            0x09 => &TRUNC_,
            0x0A => &CEIL_,
            0x0B => &FLOOR_,
            0x0C => &ROUND_,
            0x0D => &TRUNC_,
            0x0E => &CEIL_,
            0x0F => &FLOOR_,
            _ => &UNKNOWN_,
        },
    };

    Some(instruction)
}

instruction_struct!(ABS);

impl Instruction for ABS {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        if !s.cop0.cop1_usable() {
            return Some(InstructionResult::Exception(
                Exception::CoprocessorUnusable(1),
            ));
        }

        match op.cop1_format() {
            Format::S => {
                s.cpu.regs.fpr[op.fd()].set(f32::from_bits(op.ftv(s)).abs().to_bits());
            }
            Format::D => {
                s.cpu.regs.fpr[op.fd()].set64(f64::from_bits(op.fsv64(s)).abs().to_bits());
            }
            _ => unimplemented!("ABS with format {}", op.cop1_format()),
        }

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "ABS.{} {},{}",
            op.cop1_format(),
            op.fdn(),
            op.fsn()
        ))
    }
}

instruction_struct!(ADD);

impl Instruction for ADD {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        if !s.cop0.cop1_usable() {
            return Some(InstructionResult::Exception(
                Exception::CoprocessorUnusable(1),
            ));
        }

        // TODO valid modes only
        // TODO odd/even

        match op.cop1_format() {
            Format::S => {
                let ft = f32::from_bits(op.ftv(s));
                let fs = f32::from_bits(op.fsv(s));

                s.cpu.regs.fpr[op.fd()].set((ft + fs).to_bits());
            }
            Format::D => {
                let ft = f64::from_bits(op.ftv64(s));
                let fs = f64::from_bits(op.fsv64(s));

                s.cpu.regs.fpr[op.fd()].set64((ft + fs).to_bits());
            }
            _ => unimplemented!("ADD with format {}", op.cop1_format()),
        }

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "ADD.{} {}, {}, {}",
            op.cop1_format(),
            op.fdn(),
            op.fsn(),
            op.ftn()
        ))
    }
}

instruction_struct!(CEIL);

impl Instruction for CEIL {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        if !s.cop0.cop1_usable() {
            return Some(InstructionResult::Exception(
                Exception::CoprocessorUnusable(1),
            ));
        }

        match op.cop1_format() {
            Format::S => {
                s.cpu.regs.fpr[op.fd()].set(f32::from_bits(op.ftv(s)).ceil() as i32 as u32);
            }
            Format::D => {
                s.cpu.regs.fpr[op.fd()].set64(f64::from_bits(op.fsv64(s)).ceil() as i64 as u64);
            }
            _ => unimplemented!("CEIL with format {}", op.cop1_format()),
        }

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "CEIL.{} {},{}",
            op.cop1_format(),
            op.fdn(),
            op.fsn()
        ))
    }
}

instruction_struct!(CFC1);

impl Instruction for CFC1 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        // This instruction is only defined when fs is 0 or 31
        assert!(op.fs() == 31); // TODO 0 too?

        if !s.cop0.cop1_usable() {
            return Some(InstructionResult::Exception(
                Exception::CoprocessorUnusable(1),
            ));
        }

        s.cpu.regs.gpr[op.rt()].set(op.fsv(s));

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "CFC1 {}, {}",
            op.rtn(),
            Registers::fpr_name(op.fs())
        ))
    }
}

instruction_struct!(CTC1);

impl Instruction for CTC1 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        // This instruction is only defined when fs is 0 or 31
        assert!(op.fs() == 31); // TODO 0 too?

        if !s.cop0.cop1_usable() {
            return Some(InstructionResult::Exception(
                Exception::CoprocessorUnusable(1),
            ));
        }

        s.cpu.regs.fcr = op.fsv(s);

        // TODO exceptions

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        // TODO
        Disassembly::new(format!(
            "CTC1 {}, {}",
            op.rtn(),
            Registers::fpr_name(op.fs())
        ))
    }
}

instruction_struct!(DMFC1);

impl Instruction for DMFC1 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        if !s.cop0.cop1_usable() {
            return Some(InstructionResult::Exception(
                Exception::CoprocessorUnusable(1),
            ));
        }

        let value = if s.cop0.f64() {
            cop1::get64_64mode(&s.cpu.regs.fpr, op.fs())
        } else {
            cop1::get64_32mode(&s.cpu.regs.fpr, op.fs())
        };

        s.cpu.regs.gpr[op.rt()].set64(value);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DMFC1 {}, {}", op.rtn(), op.fsn()))
    }
}

instruction_struct!(DMTC1);

impl Instruction for DMTC1 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        if !s.cop0.cop1_usable() {
            return Some(InstructionResult::Exception(
                Exception::CoprocessorUnusable(1),
            ));
        }

        let rt = op.rtv64(s);

        if s.cop0.f64() {
            cop1::set64_64mode(&mut s.cpu.regs.fpr, op.fs(), rt);
        } else {
            cop1::set64_32mode(&mut s.cpu.regs.fpr, op.fs(), rt);
        }

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DMTC1 {}, {}", op.rtn(), op.fsn()))
    }
}

instruction_struct!(FLOOR);

impl Instruction for FLOOR {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        if !s.cop0.cop1_usable() {
            return Some(InstructionResult::Exception(
                Exception::CoprocessorUnusable(1),
            ));
        }

        match op.cop1_format() {
            Format::S => {
                s.cpu.regs.fpr[op.fd()].set(f32::from_bits(op.ftv(s)).floor() as i32 as u32);
            }
            Format::D => {
                s.cpu.regs.fpr[op.fd()].set64(f64::from_bits(op.fsv64(s)).floor() as i64 as u64);
            }
            _ => unimplemented!("FLOOR with format {}", op.cop1_format()),
        }

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "FLOOR.{} {},{}",
            op.cop1_format(),
            op.fdn(),
            op.fsn()
        ))
    }
}

instruction_struct!(MFC1);

impl Instruction for MFC1 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        if !s.cop0.cop1_usable() {
            return Some(InstructionResult::Exception(
                Exception::CoprocessorUnusable(1),
            ));
        }

        let value = if s.cop0.f64() {
            cop1::get32_64mode(&s.cpu.regs.fpr, op.fs())
        } else {
            cop1::get32_32mode(&s.cpu.regs.fpr, op.fs())
        };

        s.cpu.regs.gpr[op.rt()].set(value);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("MFC1 {}, {}", op.rtn(), op.fsn())) // TODO FPreg!
    }
}

instruction_struct!(MOV);

impl Instruction for MOV {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        if !s.cop0.cop1_usable() {
            return Some(InstructionResult::Exception(
                Exception::CoprocessorUnusable(1),
            ));
        }

        match op.cop1_format() {
            Format::S => {
                s.cpu.regs.fpr[op.fd()].set(op.ftv(s));
            }
            Format::D => {
                s.cpu.regs.fpr[op.fd()].set64(op.fsv64(s));
            }
            _ => unimplemented!("MOV with format {}", op.cop1_format()),
        }

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "MOV.{} {},{}",
            op.cop1_format(),
            op.fdn(),
            op.fsn()
        ))
    }
}

instruction_struct!(MTC1);

impl Instruction for MTC1 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        if !s.cop0.cop1_usable() {
            return Some(InstructionResult::Exception(
                Exception::CoprocessorUnusable(1),
            ));
        }

        let rt = op.rtv(s);
        let fs = op.fs();

        if s.cop0.f64() {
            cop1::set32_64mode(&mut s.cpu.regs.fpr, fs, rt);
        } else {
            cop1::set32_32mode(&mut s.cpu.regs.fpr, fs, rt);
        }

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("MTC1 {}, {}", op.rtn(), op.fsn()))
    }
}

instruction_struct!(NEG);

impl Instruction for NEG {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        if !s.cop0.cop1_usable() {
            return Some(InstructionResult::Exception(
                Exception::CoprocessorUnusable(1),
            ));
        }

        match op.cop1_format() {
            Format::S => {
                s.cpu.regs.fpr[op.fd()].set((-f32::from_bits(op.ftv(s))).to_bits());
            }
            Format::D => {
                s.cpu.regs.fpr[op.fd()].set64((-f64::from_bits(op.fsv64(s))).to_bits());
            }
            _ => unimplemented!("NEG with format {}", op.cop1_format()),
        }

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "NEG.{} {},{}",
            op.cop1_format(),
            op.fdn(),
            op.fsn()
        ))
    }
}

instruction_struct!(ROUND);

impl Instruction for ROUND {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        if !s.cop0.cop1_usable() {
            return Some(InstructionResult::Exception(
                Exception::CoprocessorUnusable(1),
            ));
        }

        match op.cop1_format() {
            Format::S => {
                s.cpu.regs.fpr[op.fd()].set(f32::from_bits(op.ftv(s)).round().to_bits());
            }
            Format::D => {
                s.cpu.regs.fpr[op.fd()].set64(f64::from_bits(op.fsv64(s)).round().to_bits());
            }
            _ => unimplemented!("ROUND with format {}", op.cop1_format()),
        }

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "ROUND.{} {},{}",
            op.cop1_format(),
            op.fdn(),
            op.fsn()
        ))
    }
}

instruction_struct!(SUB);

impl Instruction for SUB {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        if !s.cop0.cop1_usable() {
            return Some(InstructionResult::Exception(
                Exception::CoprocessorUnusable(1),
            ));
        }

        match op.cop1_format() {
            Format::S => {
                let ft = f32::from_bits(op.ftv(s));
                let fs = f32::from_bits(op.fsv(s));

                s.cpu.regs.fpr[op.fd()].set((fs - ft).to_bits());
            }
            Format::D => {
                let ft = f64::from_bits(op.ftv64(s));
                let fs = f64::from_bits(op.fsv64(s));

                s.cpu.regs.fpr[op.fd()].set64((fs - ft).to_bits());
            }
            _ => unimplemented!("SUB with format {}", op.cop1_format()),
        }

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "SUB.{} {}, {}, {}",
            op.cop1_format(),
            op.ftn(),
            op.fsn(),
            op.fdn()
        ))
    }
}

instruction_struct!(TRUNC);

impl Instruction for TRUNC {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<InstructionResult> {
        if !s.cop0.cop1_usable() {
            return Some(InstructionResult::Exception(
                Exception::CoprocessorUnusable(1),
            ));
        }

        match op.cop1_format() {
            Format::S => {
                s.cpu.regs.fpr[op.fd()].set(f32::from_bits(op.ftv(s)).trunc() as i32 as u32);
            }
            Format::D => {
                s.cpu.regs.fpr[op.fd()].set64(f64::from_bits(op.fsv64(s)).trunc() as i64 as u64);
            }
            _ => unimplemented!("TRUNC with format {}", op.cop1_format()),
        }

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!(
            "TRUNC.{} {},{}",
            op.cop1_format(),
            op.fdn(),
            op.fsn()
        ))
    }
}
