#![allow(clippy::upper_case_acronyms)]

use crate::map::address_info;
use crate::system::System;
pub use opcode::Opcode;

mod instructions_cop0;
mod instructions_cop1;
mod instructions_cpu;
mod opcode;

/// Result of a branch/jump: target PC to use after the delay slot.
#[derive(Clone, Copy, Debug)]
pub struct DelayedBranching(pub u32);
// TODO impl delay slot skip here?
// TODO also eret to avoid offset hack?

#[derive(Clone)]
pub struct Disassembly {
    pub mnemonics: String,
    pub hint: Option<String>,
}

impl Disassembly {
    pub fn new(mnemonics: String) -> Self {
        Self {
            mnemonics,
            hint: None,
        }
    }

    pub fn with_hint(self, hint: String) -> Self {
        Self {
            hint: Some(hint),
            ..self
        }
    }

    pub fn with_address_hint(self, addr: u32) -> Self {
        if let Some(hint) = address_info(addr) {
            Self {
                hint: Some(hint.to_string()),
                ..self
            }
        } else {
            self
        }
    }
}

/// Instruction trait.
pub trait Instruction {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching>;
    fn disassemble(&self, s: &System, op: Opcode) -> Disassembly;
}

pub fn decode(opcode: Opcode) -> Option<&'static dyn Instruction> {
    match opcode.group() {
        0b000000 => instructions_cpu::decode_special(opcode),
        0b000001 => instructions_cpu::decode_regimm(opcode),
        0b010000 => instructions_cop0::decode(opcode),
        0b010001 => instructions_cop1::decode(opcode),
        _ => instructions_cpu::decode_standard(opcode),
    }
}

/// Macro to define an instruction struct alongside a static instance (with a _ suffix).
#[macro_export]
macro_rules! instruction_struct {
    ($NAME:ident) => {
        paste::paste! {
            pub struct $NAME;
            pub static [< $NAME _ >]: $NAME = $NAME;
        }
    };
}

instruction_struct!(UNKNOWN);

impl Instruction for UNKNOWN {
    fn execute(&self, s: &mut System, op: Opcode) -> Option<DelayedBranching> {
        unimplemented!("Unknown opcode {:10X} @ {:10X}", op.0, s.cpu.regs.pc)
    }

    fn disassemble(&self, _s: &System, op: Opcode) -> Disassembly {
        Disassembly::new(format!("<UNKNOWN {:10X}>", op.0))
    }
}
