pub struct Breakpoints {
    pub breakpoints: Vec<Breakpoint>,
}

impl Breakpoints {
    pub fn new() -> Self {
        Self {
            breakpoints: Vec::new(),
        }
    }

    pub fn add(&mut self, breakpoint: Breakpoint) {
        self.breakpoints.push(breakpoint);
    }

    pub fn contains(&self, address: u64) -> bool {
        self.breakpoints.iter().any(|breakpoint| match breakpoint {
            Breakpoint::Address(bp_address) => *bp_address == address,
        })
    }
}

pub enum Breakpoint {
    Address(u64),
}
