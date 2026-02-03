use std::{cmp::Ordering, collections::BinaryHeap};

use crate::{pi::Pi, system::System};

pub type Cycle = usize;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum EventType {
    PiDmaTransferComplete,
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
            EventType::PiDmaTransferComplete => {
                log::warn!("PI DMA transfer complete");
                Pi::dma_completed(s);
            }
        }
    }
}

#[derive(Default)]
pub struct Events {
    pub events: BinaryHeap<Event>,
}

impl Events {
    pub fn push(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn update(s: &mut System) {
        while let Some(event) = s.events.pop_if_ready(s.cycles) {
            event.handle(s);
        }
    }

    fn pop_if_ready(&mut self, now: Cycle) -> Option<Event> {
        let event = self.events.peek()?;

        if event.cycle <= now {
            self.events.pop()
        } else {
            None
        }
    }

    // pub fn update(&mut self) {
    //     while let Some(event) = self.events.peek() {
    //         if event.cycle <= self.cycles {
    //             event.handle(s);
    //             self.events.pop();
    //         } else {
    //             break;
    //         }
    //     }
    // }
}
