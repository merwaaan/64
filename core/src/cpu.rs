use crate::{
    exception::Exception,
    instructions::{InstructionEffect, Opcode, decode},
    registers::Registers,
    system::{Address, System},
};

pub const FREQUENCY: f64 = 93_750_000.0;

#[derive(Default, Copy, Clone)]
pub struct Cpu {
    pub regs: Registers,

    /// Delayed branching, two levels:
    /// - Outer Option: whether there is a delayed branching
    /// - Inner Option: whether the branch was taken
    delayed_branching: Option<Option<u32>>,

    pub cycles: usize, // TODO priv
}

impl Cpu {
    pub fn step(s: &mut System) {
        // Decode and execute the current instruction

        let instruction = s
            .read(Address::v(s.cpu.regs.pc))
            .expect("Invalid instruction address"); // TODO handle exception

        let opcode = Opcode(instruction);

        let handler = decode(opcode);

        let result = match handler {
            Some((execute, _)) => execute(s, opcode),
            None => {
                panic!(
                    "Unknown instruction {:08X} at {:08X} / {}",
                    instruction, s.cpu.regs.pc, s.cpu.cycles
                );
            }
        };

        // Advance the PC

        match result {
            Ok(Some(InstructionEffect::DelayedBranching(target))) => {
                Self::advance_pc(s);

                s.cpu.delayed_branching = Some(target);
            }
            Ok(None) => {
                Self::advance_pc(s);
            }
            Err(exception) => {
                exception.raise(s);

                // Forget about the delayed branching, we don't want the branch to be taken anymore!
                // (AFTER raising, the exception handling needs to know about it to set the BD bit of CAUSE)
                s.cpu.delayed_branching = None;
            }
        }

        // Check for external interrupts

        if Exception::check_interrupts(s) {
            s.cpu.delayed_branching = None;
        }

        // Count cycles

        s.cpu.cycles = s.cpu.cycles.wrapping_add(1);
    }

    #[inline(always)]
    fn advance_pc(s: &mut System) {
        if let Some(Some(target)) = s.cpu.delayed_branching.take() {
            s.cpu.regs.pc = target;

            // Raise an exception if the target address is unaligned

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
