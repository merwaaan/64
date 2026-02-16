use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Breakpoint {
    Address(u32),
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Breakpoints {
    pub breakpoints: Vec<Breakpoint>, // TODO priv
}

impl Breakpoints {
    pub fn add(&mut self, breakpoint: Breakpoint) {
        let address = match breakpoint {
            Breakpoint::Address(addr) => addr,
        };

        if !self.contains(address) {
            self.breakpoints.push(breakpoint);
        }
    }

    pub fn remove(&mut self, breakpoint: Breakpoint) {
        self.breakpoints.retain(|bp| bp != &breakpoint);
    }

    pub fn contains(&self, address: u32) -> bool {
        self.breakpoints.iter().any(|breakpoint| match breakpoint {
            Breakpoint::Address(bp_address) => *bp_address == address,
        })
    }
}

impl<'a> IntoIterator for &'a Breakpoints {
    type Item = &'a Breakpoint;
    type IntoIter = std::slice::Iter<'a, Breakpoint>;

    fn into_iter(self) -> Self::IntoIter {
        self.breakpoints.iter()
    }
}
