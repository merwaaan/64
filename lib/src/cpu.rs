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

    nop_count: usize,
}

impl Cpu {
    pub fn step(s: &mut System) {
        // Decode and execute the current instruction

        let instruction = s.read(s.cpu.regs.pc);

        let opcode = Opcode(instruction);

        let handler = decode(opcode);

        // if opcode.0 == 0 {
        //     s.cpu.nop_count += 1;
        // } else {
        //     s.cpu.nop_count = 0;
        // }

        // if s.cpu.step > 0x2E88000 {
        //     log::error!(
        //         "opcode: {:08X} {} @ {:08X}",
        //         opcode.0,
        //         handler.unwrap().disassemble(s, opcode).mnemonics,
        //         s.cpu.regs.pc
        //     );
        // }

        if s.cpu.nop_count > 100 {
            panic!("Nop loop at {:08X}", s.cpu.regs.pc);
        }

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

                s.cpu.delayed_branching = Some(target);
            }
            Some(InstructionResult::Exception(exception)) => {
                exception.raise(s);

                // Forget about the delayed branching, we don't want the branch to be taken anymore!
                // (AFTER raising, the exception handling needs to know about it to set the BD bit of CAUSE)
                s.cpu.delayed_branching = None;
            }
            None => {
                Self::advance_pc(s);
            }
        }

        // Check for external interrupts

        if Exception::check_interrupts(s) {
            s.cpu.delayed_branching = None;
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
