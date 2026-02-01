pub struct Registers {
    pub pc: u64,
    pub gpr: [u64; 32], // TODO r0 = always 0
    pub fpr: [f64; 32],
    pub hi: u64,
    pub lo: u64,
    pub status: u32,
    pub rev: u32,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            pc: 0,
            gpr: [0; 32],
            fpr: [0.0; 32],
            hi: 0,
            lo: 0,
            status: 0,
            rev: 0,
        }
    }
}
