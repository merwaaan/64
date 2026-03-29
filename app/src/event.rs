use n64_core::{
    breakpoints::Breakpoints,
    cop0::Cop0,
    cop1::Cop1,
    events::{Cycle, EventType},
    mi::Mi,
    pi::Pi,
    si::Si,
    tlb::Tlb,
    vi::Vi,
};

use crate::ui::{
    Status,
    widgets::{
        ai_widget::AiUpdate, cpu_widget::CpuUpdate, dp_widget::DpUpdate,
        framebuffer_widget::FramebufferUpdate, memory_widget::MemoryUpdate, sp_widget::SpUpdate,
    },
};

/// Events sent from the core thread to the UI
#[derive(Debug)]
pub enum Event {
    Status(Status),
    Memory(MemoryUpdate),
    Cpu(CpuUpdate),
    Cop0(Cop0),
    Cop1(Cop1),
    Mi(Mi),
    Vi(Vi),
    Ai(AiUpdate),
    Pi(Pi),
    Sp(SpUpdate),
    Dp(DpUpdate),
    Si(Si),
    Tlb(Tlb),
    Framebuffer(FramebufferUpdate),
    IsViewer(String),
    Breakpoints(Breakpoints),
    Events {
        current_cycle: Cycle,
        pending: Vec<(EventType, Cycle)>,
    },
}
