use crate::{
    cpu::{
        decoder::{Decoder, basic::BasicDecoder},
        instructions::InstructionEffect,
        opcode::Opcode,
    },
    exception::Exception,
    registers::Registers,
    system::{Address, System},
};

pub mod decoder;
pub mod instructions;
pub mod opcode;
mod operands;

pub const FREQUENCY: f64 = 93_750_000.0;

pub struct Cpu {
    pub regs: Registers,

    cycles: usize,

    /// Delayed branching, two levels:
    /// - Outer Option: whether there is a delayed branching
    /// - Inner Option: target address if the branch was taken
    delayed_branching: Option<Option<u32>>,

    //pub decoder: LutDecoder,
    pub decoder: BasicDecoder,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            regs: Registers::default(),
            delayed_branching: None,
            cycles: 0,
            //decoder: LutDecoder::default(), // BasicDecoder::default(),
            decoder: BasicDecoder::default(),
        }
    }
}

impl Cpu {
    pub fn cycles(&self) -> usize {
        self.cycles
    }

    pub fn step(s: &mut System) {
        // Decode and execute the current instruction

        let result = s.read(Address::v(s.cpu.regs.pc)).and_then(|instruction| {
            let opcode = Opcode(instruction);
            let decoder = s.cpu.decoder;
            decoder.execute(s, opcode)
        });

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
                Exception::AddressLoad { address: target }.raise(s);
            }
        } else {
            s.cpu.regs.pc = s.cpu.regs.pc.wrapping_add(4);
        }
    }

    pub fn in_branch_delay_slot(&self) -> bool {
        self.delayed_branching.is_some()
    }
}
