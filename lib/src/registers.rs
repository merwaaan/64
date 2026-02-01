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

    pub fn gpr_name(index: usize) -> &'static str {
        const NAMES: [&str; 32] = [
            "ZERO", "AT", "V0", "V1", "A0", "A1", "A2", "A3", "T0", "T1", "T2", "T3", "T4", "T5",
            "T6", "T7", "S0", "S1", "S2", "S3", "S4", "S5", "S6", "S7", "T8", "T9", "K0", "K1",
            "GP", "SP", "S8", "RA",
        ];

        NAMES.get(index).copied().unwrap_or("?")
    }
}
