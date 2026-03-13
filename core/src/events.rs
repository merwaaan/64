use std::{cmp::Ordering, collections::BinaryHeap};

use crate::{ai::Ai, pi::Pi, si::Si, sp::Sp, system::System, vi::Vi};

pub type Cycle = usize;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum EventType {
    AiDmaTransferComplete,
    PiDmaTransferComplete,
    SpDmaTransferComplete,
    SiDmaTransferComplete,
    ViScanlineComplete,
    // SpHalt, // TODO temp hack
    // DpHalt, // TODO temp hack
}

#[derive(Debug, Copy, Clone)]
pub struct Event {
    pub id: EventType,
    pub cycle: Cycle,
}

// TODO partial eq vs eq?
impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.cycle == other.cycle
    }
}

impl Eq for Event {}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse so that earlier time = higher priority in the max-heap.
        other.cycle.cmp(&self.cycle)
    }
}

impl Event {
    pub fn handle(&self, s: &mut System) {
        match self.id {
            EventType::AiDmaTransferComplete => {
                Ai::dma_completed(s);
            }
            EventType::PiDmaTransferComplete => {
                Pi::dma_completed(s);
            }
            EventType::SpDmaTransferComplete => {
                Sp::dma_completed(s);
            }
            EventType::SiDmaTransferComplete => {
                Si::dma_completed(s);
            }
            EventType::ViScanlineComplete => {
                Vi::scanline_completed(s);
            } // EventType::SpHalt => {
              //     Sp::halt(s);
              // }
              // EventType::DpHalt => {
              //     Sp::dp_halt(s);
              // }
        }
    }
}

#[derive(Default)]
pub struct Events {
    events: BinaryHeap<Event>,
}

impl Events {
    pub(crate) fn push(s: &mut System, event: EventType, in_cycles: Cycle) {
        s.events.events.push(Event {
            id: event,
            cycle: s.cpu.cycles() + in_cycles,
        });
    }

    pub(crate) fn update(s: &mut System) {
        while let Some(event) = s.events.pop_if_ready(s.cpu.cycles()) {
            event.handle(s);
        }
    }

    pub(crate) fn pop_if_ready(&mut self, now: Cycle) -> Option<Event> {
        let event = self.events.peek()?;

        if now >= event.cycle {
            self.events.pop()
        } else {
            None
        }
    }

    pub fn snapshot(&self) -> Vec<(EventType, Cycle)> {
        let mut v: Vec<_> = self.events.iter().map(|e| (e.id, e.cycle)).collect();
        v.sort_by_key(|&(_, c)| c);
        v
    }
}
