use n64_core::{
    ai::Ai,
    breakpoints::Breakpoints,
    events::{Cycle, EventType},
    mi::Mi,
    si::Si,
    vi::Vi,
};

use crate::ui::{
    Status, framebuffer::FramebufferUpdate, instructions::InstructionData, memory::MemoryUpdate,
    registers::RegistersUpdate,
};

pub enum Event {
    StatusUpdate(Status),
    RegistersUpdate(RegistersUpdate),
    MemoryUpdate(MemoryUpdate),
    InstructionsUpdate(Vec<InstructionData>),
    MiUpdate(Mi),
    ViUpdate(Vi),
    AiUpdate(Ai),
    SpUpdate([u32; 8]),
    SiUpdate(Si),
    FramebufferUpdate(FramebufferUpdate),
    IsViewerUpdate(String),
    BreakpointsUpdate(Breakpoints),
    CoreEventsUpdate {
        current_cycle: Cycle,
        pending: Vec<(EventType, Cycle)>,
    },
}
