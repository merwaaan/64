use crate::{
    instructions::{DelayedBranching, Opcode, decode},
    registers::Registers,
    system::System,
};

pub struct CPU {
    pub regs: Registers,

    delayed_branching: Option<DelayedBranching>,

    pub step: usize,
}

impl Default for CPU {
    fn default() -> Self {
        Self {
            regs: Registers::default(),

            delayed_branching: None,

            step: 0, // TODO move up?
        }
    }
}

impl CPU {
    pub fn step(s: &mut System) {
        let instruction = s.read(s.cpu.regs.pc);

        // if instruction == 0x74027 {
        //     panic!("PC: {:08X}", self.regs.pc);
        // }

        let opcode = Opcode(instruction);
        let handler = decode(opcode);

        let next_delayed_branching = handler.execute(s, opcode);

        match s.cpu.delayed_branching.take() {
            Some(DelayedBranching(target)) => s.cpu.regs.pc = target,
            None => s.cpu.regs.pc += 4,
        }

        s.cpu.delayed_branching = next_delayed_branching;

        s.cpu.step += 1;
    }
}
