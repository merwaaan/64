use crate::{
    cpu::{
        instructions_cop0, instructions_cop1, instructions_cop2, instructions_cpu, opcode::Opcode,
    },
    exception::Exception,
    system::System,
};

#[derive(Clone, Copy, Debug)]
pub enum InstructionEffect {
    /// The instruction was a delayed branching.
    /// If the branch was taken, contains the target address.
    DelayedBranching(Option<u32>),
    // TODO SkipDelaySlot
}

pub type InstructionResult = Result<Option<InstructionEffect>, Exception>;

pub type ExecuteFn = fn(&mut System, Opcode) -> InstructionResult;
pub type DisassembleFn = fn(&System, Opcode) -> String;
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

pub fn decode(opcode: Opcode) -> DecodedInstruction {
    match opcode.group() {
        0b000000 => instructions_cpu::decode_special(opcode),
        0b000001 => instructions_cpu::decode_regimm(opcode),
        0b010000 => instructions_cop0::decode(opcode),
        0b010001 => instructions_cop1::decode(opcode),
        0b010010 => instructions_cop2::decode(opcode),
        // Move standard here, same discriminant
        _ => instructions_cpu::decode_standard(opcode),
    }
}

// Reserved instruction

fn reserved_execute(_s: &mut System, _op: Opcode) -> InstructionResult {
    Err(Exception::ReservedInstruction)
}

pub fn reserved_disassemble(_s: &System, op: Opcode) -> String {
    format!("<RESERVED {:08X}>", op.0)
}

pub const RESERVED_INSTRUCTION: DecodedInstruction = (reserved_execute, reserved_disassemble);
