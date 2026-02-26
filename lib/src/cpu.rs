use crate::{
    instructions::{InstructionResult, Opcode, decode},
    registers::Registers,
    system::System,
};

#[derive(Default, Copy, Clone)]
pub struct CPU {
    pub regs: Registers,

    pub delayed_branching: Option<u32>,

    pub step: usize,
}

impl CPU {
    pub fn step(s: &mut System) {
        // Decode and execute the current instruction

        let instruction = s.read(s.cpu.regs.pc);

        let opcode = Opcode(instruction);

        let handler = decode(opcode);

        let instruction_result = match handler {
            Some(handler) => handler.execute(s, opcode),
            None => {
                panic!(
                    "Unknown instruction {:08X} at {:08X} / {}",
                    instruction, s.cpu.regs.pc, s.cpu.step
                );
            }
        };

        // Advance the PC

        match instruction_result {
            Some(InstructionResult::DelayedBranching(target)) => {
                s.cpu.advance_pc();

                s.cpu.delayed_branching = Some(target);
            }
            Some(InstructionResult::Exception(exception)) => {
                exception.raise(s);

                // Forget about the delayed branching
                // (AFTER raising, the exception code needs to access it)
                s.cpu.delayed_branching = None;
            }
            None => {
                s.cpu.advance_pc();
            }
        }

        // TODO rm

        s.cpu.step += 1;
    }

    fn advance_pc(&mut self) {
        if let Some(target) = self.delayed_branching.take() {
            self.regs.pc = target;
        } else {
            self.regs.pc = self.regs.pc.wrapping_add(4);
        }
    }
}
