use crate::exception::Exception;
use crate::map::address_info;
use crate::system::System;
pub use opcode::Opcode;

mod instructions_cop0;
mod instructions_cop1;
mod instructions_cop2;
mod instructions_cpu;
mod opcode;

#[derive(Clone, Copy, Debug)]
pub enum InstructionResult {
    /// The instruction was a delayed branching.
    /// If the branch was taken, contains the target address.
    DelayedBranching(Option<u32>),
    /// The instruction caused an exception
    Exception(Exception),
}

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

pub type ExecuteFn = fn(&mut System, Opcode) -> Option<InstructionResult>;
pub type DisassembleFn = fn(&System, Opcode) -> Disassembly;
pub type DecodedInstruction = (ExecuteFn, DisassembleFn);

/// Expands to `(name_execute, name_disassemble)` for use in decode match arms.
#[macro_export]
macro_rules! inst {
    ($name:ident) => {
        (
            paste::paste! { [< $name _execute >] },
            paste::paste! { [< $name _disassemble >] },
        )
    };
}

pub fn decode(opcode: Opcode) -> Option<DecodedInstruction> {
    match opcode.group() {
        0b000000 => instructions_cpu::decode_special(opcode),
        0b000001 => instructions_cpu::decode_regimm(opcode),
        0b010000 => instructions_cop0::decode(opcode),
        0b010001 => instructions_cop1::decode(opcode),
        0b010010 => instructions_cop2::decode(opcode),
        _ => instructions_cpu::decode_standard(opcode),
    }
}
