use n64::registers::Registers;

use crate::ui::instructions::InstructionData;

pub enum Event {
    Update {
        cpu_regs: Option<Registers>,
        memory: Option<Vec<u32>>,
        instructions: Option<Vec<InstructionData>>,
    },
}
