use crate::{
    cpu::{instructions::InstructionEffect, opcode::Opcode},
    exception::Exception,
    registers::Registers,
    system::{Address, System},
};

pub mod instructions;
pub(crate) mod instructions_cop0;
pub(crate) mod instructions_cop1;
pub(crate) mod instructions_cop2;
pub(crate) mod instructions_cpu;
pub mod opcode;

pub const FREQUENCY: f64 = 93_750_000.0;

#[derive(Default, Copy, Clone, Debug)]
pub struct Cpu {
    pub regs: Registers,

    /// Delayed branching, two levels:
    /// - Outer Option: whether there is a delayed branching
    /// - Inner Option: whether the branch was taken
    delayed_branching: Option<Option<u32>>,

    cycles: usize,
}

impl Cpu {
    pub fn cycles(&self) -> usize {
        self.cycles
    }

    pub fn step(s: &mut System) {
        // Decode and execute the current instruction

        let instruction = s
            .read(Address::v(s.cpu.regs.pc))
            .unwrap_or_else(|_| panic!("Invalid instruction address {:08X}", s.cpu.regs.pc)); // TODO handle exception

        let opcode = Opcode(instruction);

        let (execute, _disassemble) = instructions::decode(opcode);

        let result = execute(s, opcode);

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
