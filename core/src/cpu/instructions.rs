use crate::{
    cpu::{opcode::Opcode, operands::Operands},
    exception::Exception,
    system::System,
};

pub mod cop0;
pub mod cop1;
pub mod cop2;
pub mod regimm;
pub mod special;
pub mod standard;

/// Effect of an executed instruction.
#[derive(Clone, Copy, Debug)]
pub enum InstructionEffect {
    /// The instruction is a delayed branching.
    /// If the branch is taken, contains the target address.
    DelayedBranching(Option<u32>),
    // TODO SkipDelaySlot
}

pub type InstructionResult = Result<Option<InstructionEffect>, Exception>;

pub trait Instruction {
    fn execute(s: &mut System, opcode: Opcode, operands: Operands) -> InstructionResult;
    fn disassemble(s: &System, opcode: Opcode, operands: Operands) -> String;
}

pub type ExecuteFn = fn(&mut System, Opcode, Operands) -> InstructionResult;
pub type DisassembleFn = fn(&System, Opcode, Operands) -> String;
pub type DecodedInstruction = (ExecuteFn, DisassembleFn);

// TODO move out?
#[macro_export]
macro_rules! inst {
    ($name:ident) => {
        (
            paste::paste! { [< $name _execute >] },
            paste::paste! { [< $name _disassemble >] },
        )
    };
}

// Reserved instruction

// TODO mips manual p 544, not all invalid opcodes cause exceptions?

pub struct Reserved;

impl Instruction for Reserved {
    fn execute(_s: &mut System, opcode: Opcode, _operands: Operands) -> InstructionResult {
        log::warn!("Reserved instruction: {:08X}", opcode.0);

        Err(Exception::ReservedInstruction)
    }

    fn disassemble(_s: &System, opcode: Opcode, _operands: Operands) -> String {
        format!("<RESERVED {:08X}>", opcode.0)
    }
}

pub const RESERVED_INSTRUCTION: DecodedInstruction = (
    <Reserved as Instruction>::execute,
    <Reserved as Instruction>::disassemble,
);

// Helpers

fn trap(condition: bool) -> InstructionResult {
    if condition {
        Err(Exception::Trap)
    } else {
        Ok(None)
    }
}

fn branch<const DISCARD_DELAY_SLOT: bool>(
    s: &mut System,
    op: Opcode,
    condition: bool,
) -> InstructionResult {
    Ok(Some(InstructionEffect::DelayedBranching(if condition {
        Some(op.branch_target(s))
    } else {
        // Discard the instruction in the delay slot TODO return special val??
        if DISCARD_DELAY_SLOT {
            s.cpu.regs.pc = s.cpu.regs.pc.wrapping_add(4);
        }

        None
    })))
}
