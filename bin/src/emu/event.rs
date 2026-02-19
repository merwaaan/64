use n64::{ai::Ai, breakpoints::Breakpoints, mi::Mi, rsp::Rsp, si::Si, vi::Vi};

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
    AiUpdate(Ai),
    RspUpdate([u32; 8]),
    SiUpdate(Si),
    FramebufferUpdate(FramebufferUpdate),
    BreakpointsUpdate(Breakpoints),
}
