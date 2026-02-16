use n64::{breakpoints::Breakpoints, mi::Mi, vi::Vi};

use crate::ui::{
    framebuffer::FramebufferUpdate, instructions::InstructionData, memory::MemoryUpdate,
    registers::RegistersUpdate,
};

pub enum Event {
    Pause,
    RegistersUpdate(RegistersUpdate),
    MemoryUpdate(MemoryUpdate),
    InstructionsUpdate(Vec<InstructionData>),
    MiUpdate(Mi),
    ViUpdate(Vi),
    FramebufferUpdate(FramebufferUpdate),
    BreakpointsUpdate(Breakpoints),
}
