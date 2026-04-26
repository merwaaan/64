use crate::{
    cpu::{instructions::InstructionResult, opcode::Opcode},
    system::System,
};

pub mod basic;
pub mod lut;

pub trait Decoder {
    fn execute(&self, s: &mut System, op: Opcode) -> InstructionResult;
    fn disassemble(&self, s: &System, op: Opcode) -> String;
}
