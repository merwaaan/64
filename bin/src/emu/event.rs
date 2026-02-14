use n64::registers::Registers;

use crate::ui::instructions::InstructionData;

pub enum Event {
    Update {
        cpu_regs: Registers,
        memory: Vec<u8>,
        instructions: Vec<InstructionData>,
    },
}
