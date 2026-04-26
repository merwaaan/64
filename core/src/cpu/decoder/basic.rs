use crate::{
    cpu::{
        decoder::Decoder,
        instructions::{DecodedInstruction, InstructionResult},
        opcode::Opcode,
        operands::Operands,
    },
    decode_cop0_x, decode_cop1_inst_fmt, decode_cop1_inst_fmt_fp, decode_cop1_x, decode_cop2_x,
    decode_regimm_x, decode_special_x, decode_standard_x,
    system::System,
};

/// Basic instruction decoder that uses nested matches to retrieve the execute and disassemble functions for the given opcode.
#[derive(Clone, Copy, Default)]
pub struct BasicDecoder;

impl BasicDecoder {
    fn decoded(&self, opcode: Opcode) -> DecodedInstruction {
        macro_rules! exec_disasm {
            ($ty:path) => {
                (
                    <$ty as $crate::cpu::instructions::Instruction>::execute,
                    <$ty as $crate::cpu::instructions::Instruction>::disassemble,
                )
            };
        }

        match opcode.group() {
            0b000000 => decode_special_x!(opcode, exec_disasm),
            0b000001 => decode_regimm_x!(opcode, exec_disasm),
            0b010000 => decode_cop0_x!(opcode, exec_disasm),
            0b010001 => decode_cop1_x!(opcode, exec_disasm),
            0b010010 => decode_cop2_x!(opcode, exec_disasm),
            _ => decode_standard_x!(opcode, exec_disasm),
        }
    }
}

impl Decoder for BasicDecoder {
    fn execute(&self, s: &mut System, op: Opcode) -> InstructionResult {
        let (execute, _) = self.decoded(op);
        let operands = Operands::from_opcode(op);
        execute(s, op, operands)
    }

    fn disassemble(&self, s: &System, op: Opcode) -> String {
        let (_, disassemble) = self.decoded(op);
        let operands = Operands::from_opcode(op);
        disassemble(s, op, operands)
    }
}
