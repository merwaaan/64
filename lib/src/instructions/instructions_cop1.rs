#![allow(clippy::upper_case_acronyms)]

use super::{DelayedBranching, Disassembly, Instruction, Opcode, System};
use crate::{instruction_struct, registers::Registers};

/// COP1 rs field (bits 25–21).
fn cop1_rs(opcode: Opcode) -> u32 {
    (opcode.0 >> 21) & 0x1F
}

/// COP1 function field (bits 5–0) for S/D/W/L formats.
fn cop1_func(opcode: Opcode) -> u32 {
    opcode.0 & 0x3F
}

pub fn decode(opcode: Opcode) -> Option<&'static dyn Instruction> {
    debug_assert_eq!(opcode.group(), 0x11);

    let instruction: &'static dyn Instruction = match cop1_rs(opcode) {
        0x00 => &MFC1_,
        //0x01 => &DMFC1_,
        0x02 => &CFC1_,
        //0x03 => &DCFC1_,
        0x04 => &MTC1_,
        //0x05 => &DMTC1_,
        0x06 => &CTC1_,
        //0x07 => &DCTC1_,
        //0x08 => &BC1_,
        //0x09 => &COP1_S_,
        //0x0A => &COP1_D_,
        // 0x0B => &COP1_W_,
        //0x0C => &COP1_L_,
        _ => &COP1_RESERVED_,
    };

    Some(instruction)
}

instruction_struct!(CFC1);

impl Instruction for CFC1 {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        assert!(op.fs() == 31); // TODO 0 too?

        s.cpu.regs.gpr[op.rt()].set(op.fsv(s));

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
        assert!(op.fs() == 31); // TODO 0 too?

        s.cpu.regs.fcr = op.fsv(s);

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
        panic!("MFC1");

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("MFC1 {}, {}", op.rtn(), op.rd0n())) // TODO FPreg!
    }
}

instruction_struct!(MTC1);

impl Instruction for MTC1 {
    fn execute(&self, _s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        panic!("MFC1");

        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("MTC1 {}, {}", op.rtn(), op.rd0n()))
    }
}

// --- Stubs for COP1 instructions not yet implemented ---

// instruction_struct!(DMFC1);

// impl Instruction for DMFC1 {
//     fn execute(&self, _s: &mut System, _op: Opcode) -> Option<DelayedBranching> {
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
//     fn execute(&self, _s: &mut System, _op: Opcode) -> Option<DelayedBranching> {
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
//     fn execute(&self, _s: &mut System, _op: Opcode) -> Option<DelayedBranching> {
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
//     fn execute(&self, _s: &mut System, _op: Opcode) -> Option<DelayedBranching> {
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
//     fn execute(&self, _s: &mut System, _op: Opcode) -> Option<DelayedBranching> {
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

instruction_struct!(COP1_RESERVED);

impl Instruction for COP1_RESERVED {
    fn execute(&self, _s: &mut System, _op: Opcode) -> Option<DelayedBranching> {
        log::debug!("COP1 reserved (stub)");
        None
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("<COP1 reserved rs={}>", cop1_rs(op)))
    }
}

fn cop1_s_mnemonic(func: u32) -> &'static str {
    match func {
        0x00 => "ADD",
        0x01 => "SUB",
        0x02 => "MUL",
        0x03 => "DIV",
        0x04 => "SQRT",
        0x05 => "ABS",
        0x06 => "MOV",
        0x07 => "NEG",
        0x08 => "ROUND.L",
        0x09 => "TRUNC.L",
        0x0A => "CEIL.L",
        0x0B => "FLOOR.L",
        0x0C => "ROUND.W",
        0x0D => "TRUNC.W",
        0x0E => "CEIL.W",
        0x0F => "FLOOR.W",
        0x20 => "CVT.S",
        0x21 => "CVT.D",
        0x24 => "CVT.W",
        0x25 => "CVT.L",
        0x30 => "C.F",
        0x31 => "C.UN",
        0x32 => "C.EQ",
        0x33 => "C.UEQ",
        0x34 => "C.OLT",
        0x35 => "C.ULT",
        0x36 => "C.OLE",
        0x37 => "C.ULE",
        0x38 => "C.SF",
        0x39 => "C.NGLE",
        0x3A => "C.SEQ",
        0x3B => "C.NGL",
        0x3C => "C.LT",
        0x3D => "C.NGE",
        0x3E => "C.LE",
        0x3F => "C.NGT",
        _ => "COP1.S",
    }
}

fn cop1_d_mnemonic(func: u32) -> &'static str {
    match func {
        0x00 => "ADD",
        0x01 => "SUB",
        0x02 => "MUL",
        0x03 => "DIV",
        0x04 => "SQRT",
        0x05 => "ABS",
        0x06 => "MOV",
        0x07 => "NEG",
        0x08 => "ROUND.L",
        0x09 => "TRUNC.L",
        0x0A => "CEIL.L",
        0x0B => "FLOOR.L",
        0x0C => "ROUND.W",
        0x0D => "TRUNC.W",
        0x0E => "CEIL.W",
        0x0F => "FLOOR.W",
        0x20 => "CVT.S",
        0x21 => "CVT.D",
        0x24 => "CVT.W",
        0x25 => "CVT.L",
        0x30 => "C.F",
        0x31 => "C.UN",
        0x32 => "C.EQ",
        0x33 => "C.UEQ",
        0x34 => "C.OLT",
        0x35 => "C.ULT",
        0x36 => "C.OLE",
        0x37 => "C.ULE",
        0x38 => "C.SF",
        0x39 => "C.NGLE",
        0x3A => "C.SEQ",
        0x3B => "C.NGL",
        0x3C => "C.LT",
        0x3D => "C.NGE",
        0x3E => "C.LE",
        0x3F => "C.NGT",
        _ => "COP1.D",
    }
}

fn cop1_wl_mnemonic(func: u32) -> &'static str {
    match func {
        0x00 => "ADD",
        0x01 => "SUB",
        0x02 => "MUL",
        0x03 => "DIV",
        0x04 => "SQRT",
        0x05 => "ABS",
        0x06 => "MOV",
        0x07 => "NEG",
        0x20 => "CVT.S",
        0x21 => "CVT.D",
        0x24 => "CVT.W",
        0x25 => "CVT.L",
        _ => "COP1",
    }
}
