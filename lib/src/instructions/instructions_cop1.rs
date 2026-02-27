#![allow(clippy::upper_case_acronyms)]

use super::{Disassembly, Instruction, InstructionResult, Opcode, System};
use crate::{
    exception::Exception, instruction_struct, instructions::UNKNOWN_, registers::Registers,
};

/// COP1 rs field (bits 25–21).
fn cop1_rs(opcode: Opcode) -> u32 {
    (opcode.0 >> 21) & 0x1F
}

pub fn decode(opcode: Opcode) -> Option<&'static dyn Instruction> {
    debug_assert_eq!(opcode.group(), 0x11);

    let instruction: &'static dyn Instruction = match cop1_rs(opcode) {
        0x00 => &MFC1_,
        0x01 => &DMFC1_,
        0x02 => &CFC1_,
        //0x03 => &DCFC1_,
        0x04 => &MTC1_,
        0x05 => &DMTC1_,
        0x06 => &CTC1_,
        //0x07 => &DCTC1_,
        //0x08 => &BC1_,
        //0x09 => &COP1_S_,
        //0x0A => &COP1_D_,
        // 0x0B => &COP1_W_,
        //0x0C => &COP1_L_,
        _ => &UNKNOWN_,
    };

    Some(instruction)
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

        let freg = op.fs();

        if s.cop0.f_64() {
            s.cpu.regs.gpr[op.rt()].set64(s.cpu.regs.fpr[freg].get64());
        } else {
            s.cpu.regs.gpr[op.rt()].set64(s.cpu.regs.fpr[freg & !1].get64());
        }

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

        let freg = op.fs();

        if s.cop0.f_64() {
            s.cpu.regs.fpr[freg].set64(op.rtv64(s));
        } else {
            s.cpu.regs.fpr[freg & !1].set64(op.rtv64(s));
        }

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("DMTC1 {}, {}", op.rtn(), op.rd0n()))
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

        let value = if s.cop0.f_64() || op.fs() & 1 == 0 {
            op.fsv(s)
        } else {
            (s.cpu.regs.fpr[op.fs() & !1].get64() >> 32) as u32
        };

        s.cpu.regs.gpr[op.rt()].set(value);

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("MFC1 {}, {}", op.rtn(), op.rd0n())) // TODO FPreg!
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
        Disassembly::new(format!("MTC1 {}, {}", op.rtn(), op.rd0n()))
    }
}

// --- Stubs for COP1 instructions not yet implemented ---

// instruction_struct!(DMFC1);

// impl Instruction for DMFC1 {
//     fn execute(&self, _s: &mut System, _op: Opcode) -> Option<InstructionResult> {
//         log::debug!("DMFC1 (stub)");
//         None
//     }

//     fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
//         Disassembly::new(format!(
//             "DMFC1 {}, {}",
//             op.rtn(),
//             Registers::fpr_name(op.rd())
//         ))
//     }
// }

// instruction_struct!(DCFC1);

// impl Instruction for DCFC1 {
//     fn execute(&self, _s: &mut System, _op: Opcode) -> Option<InstructionResult> {
//         log::debug!("DCFC1 (stub)");
//         None
//     }

//     fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
//         Disassembly::new(format!(
//             "DCFC1 {}, {}",
//             op.rtn(),
//             Registers::fpr_name(op.rd())
//         ))
//     }
// }

// instruction_struct!(DMTC1);

// impl Instruction for DMTC1 {
//     fn execute(&self, _s: &mut System, _op: Opcode) -> Option<InstructionResult> {
//         log::debug!("DMTC1 (stub)");
//         None
//     }

//     fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
//         Disassembly::new(format!(
//             "DMTC1 {}, {}",
//             op.rtn(),
//             Registers::fpr_name(op.rd())
//         ))
//     }
// }

// instruction_struct!(DCTC1);

// impl Instruction for DCTC1 {
//     fn execute(&self, _s: &mut System, _op: Opcode) -> Option<InstructionResult> {
//         log::debug!("DCTC1 (stub)");
//         None
//     }

//     fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
//         Disassembly::new(format!(
//             "DCTC1 {}, {}",
//             op.rtn(),
//             Registers::fpr_name(op.rd())
//         ))
//     }
// }

// instruction_struct!(BC1);

// impl Instruction for BC1 {
//     fn execute(&self, _s: &mut System, _op: Opcode) -> Option<InstructionResult> {
//         log::debug!("BC1 (stub)");
//         None
//     }

//     fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
//         let mnemonic = match op.rt() {
//             0 => "BC1F",
//             1 => "BC1T",
//             2 => "BC1FL",
//             3 => "BC1TL",
//             _ => "BC1?",
//         };
//         Disassembly::new(format!("{} {:#X}", mnemonic, op.branch_offset()))
//     }
// }

// instruction_struct!(COP1_S);

// impl Instruction for COP1_S {
//     fn execute(&self, _s: &mut System, _op: Opcode) -> Option<DelayedBranching> {
//         log::debug!("COP1.S (stub)");
//         None
//     }

//     fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
//         let mnemonic = cop1_s_mnemonic(cop1_func(op));
//         Disassembly::new(format!(
//             "{}.S {}, {}, {}",
//             mnemonic,
//             Registers::fpr_name(op.rd()),
//             Registers::fpr_name(op.rs()),
//             Registers::fpr_name(op.rt())
//         ))
//     }
// }

// instruction_struct!(COP1_D);

// impl Instruction for COP1_D {
//     fn execute(&self, _s: &mut System, _op: Opcode) -> Option<DelayedBranching> {
//         log::debug!("COP1.D (stub)");
//         None
//     }

//     fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
//         let mnemonic = cop1_d_mnemonic(cop1_func(op));
//         Disassembly::new(format!(
//             "{}.D {}, {}, {}",
//             mnemonic,
//             Registers::fpr_name(op.rd()),
//             Registers::fpr_name(op.rs()),
//             Registers::fpr_name(op.rt())
//         ))
//     }
// }

// instruction_struct!(COP1_W);

// impl Instruction for COP1_W {
//     fn execute(&self, _s: &mut System, _op: Opcode) -> Option<DelayedBranching> {
//         log::debug!("COP1.W (stub)");
//         None
//     }

//     fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
//         let mnemonic = cop1_wl_mnemonic(cop1_func(op));
//         Disassembly::new(format!(
//             "{}.W {}, {}",
//             mnemonic,
//             Registers::fpr_name(op.rd()),
//             Registers::fpr_name(op.rs())
//         ))
//     }
// }

// instruction_struct!(COP1_L);

// impl Instruction for COP1_L {
//     fn execute(&self, _s: &mut System, _op: Opcode) -> Option<DelayedBranching> {
//         log::debug!("COP1.L (stub)");
//         None
//     }

//     fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
//         let mnemonic = cop1_wl_mnemonic(cop1_func(op));
//         Disassembly::new(format!(
//             "{}.L {}, {}",
//             mnemonic,
//             Registers::fpr_name(op.rd()),
//             Registers::fpr_name(op.rs())
//         ))
//     }
// }

// instruction_struct!(COP1_RESERVED);

// impl Instruction for COP1_RESERVED {
//     fn execute(&self, _s: &mut System, _op: Opcode) -> Option<DelayedBranching> {
//         log::debug!("COP1 reserved (stub)");
//         None
//     }

//     fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
//         Disassembly::new(format!("<COP1 reserved rs={}>", cop1_rs(op)))
//     }
// }
