use crate::{
    exception::Exception,
    instructions::{InstructionResult, Opcode, decode},
    registers::Registers,
    system::System,
};

#[derive(Default, Copy, Clone)]
pub struct Cpu {
    pub regs: Registers,

    /// Delayed branching, two levels:
    /// - Outer Option: whether there is a delayed branching
    /// - Inner Option: whether the branch was taken
    delayed_branching: Option<Option<u32>>,

    pub step: usize,
}

impl Cpu {
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
                Self::advance_pc(s);

                s.cpu.delayed_branching = Some(target.clone());
            }
            Some(InstructionResult::Exception(exception)) => {
                exception.raise(s);

                // Forget about the delayed branching
                // (AFTER raising, the exception handling depends on it)
                s.cpu.delayed_branching = None;
            }
            None => {
                Self::advance_pc(s);
            }
        }

        // TODO rm

        s.cpu.step += 1;
    }

    #[inline(always)]
    fn advance_pc(s: &mut System) {
        if let Some(Some(target)) = s.cpu.delayed_branching.take() {
            s.cpu.regs.pc = target;

            // Handle unaligned target addresses

            if target & 3 != 0 {
                Exception::AddressLoad(target).raise(s);
            }
        } else {
            s.cpu.regs.pc = s.cpu.regs.pc.wrapping_add(4);
        }
    }

    pub fn in_branch_delay_slot(&self) -> bool {
        self.delayed_branching.is_some()
    }
}
