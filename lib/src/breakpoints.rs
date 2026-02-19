use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
struct Breakpoint {
    enabled: bool,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Breakpoints {
    breakpoints: HashMap<u32, Breakpoint>,
}

impl Breakpoints {
    pub fn add(&mut self, address: u32) {
        self.breakpoints
            .insert(address, Breakpoint { enabled: true });
    }

    pub fn remove(&mut self, address: u32) {
        self.breakpoints.remove(&address);
    }

    pub fn toggle(&mut self, address: u32) {
        self.breakpoints
            .get_mut(&address)
            .map(|breakpoint| breakpoint.enabled = !breakpoint.enabled);
    }

    pub fn should_break(&self, address: u32) -> bool {
        self.breakpoints.get(&address).is_some_and(|b| b.enabled) // TODO if None, return true?b
    }

    pub fn iter(&self) -> impl Iterator<Item = (u32, bool)> {
        self.breakpoints
            .iter()
            .map(|(address, breakpoint)| (*address, breakpoint.enabled))
    }
}
