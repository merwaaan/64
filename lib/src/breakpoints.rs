#[derive(Default)]
pub struct Breakpoints {
    pub breakpoints: Vec<Breakpoint>, // TODO priv
}

impl Breakpoints {
    pub fn add(&mut self, breakpoint: Breakpoint) {
        // TODO unique
        self.breakpoints.push(breakpoint);
    }

    pub fn contains(&self, address: u32) -> bool {
        self.breakpoints.iter().any(|breakpoint| match breakpoint {
            Breakpoint::Address(bp_address) => *bp_address == address,
        })
    }
}

pub enum Breakpoint {
    Address(u32),
}
