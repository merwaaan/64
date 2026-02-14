use crate::{
    instructions::{DelayedBranching, Opcode, decode},
    registers::Registers,
    system::System,
};

#[derive(Default, Copy, Clone)]
pub struct CPU {
    pub regs: Registers,

    delayed_branching: Option<DelayedBranching>,

    pub step: usize,
}

impl CPU {
    pub fn step(s: &mut System) {
        // Decode and execute the current instruction

        let instruction = s.read(s.cpu.regs.pc);

        let opcode = Opcode(instruction);

        let handler = decode(opcode);

        let next_delayed_branching = handler.execute(s, opcode);

        // Advance the PC

        match s.cpu.delayed_branching.take() {
            Some(DelayedBranching(target)) => s.cpu.regs.pc = target,
            None => s.cpu.regs.pc += 4,
        }

        s.cpu.delayed_branching = next_delayed_branching;

        // TODO rm

        s.cpu.step += 1;
    }
}
