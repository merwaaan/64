#[derive(Default)]
pub struct Registers {
    pub pc: u32,
    pub gpr: [u32; 32], // TODO r0 = always 0
    pub fpr: [f32; 32], // TODO 64 or 32?
    pub mult_hi: u32,
    pub mult_lo: u32,
    pub status: u32,
    pub rev: u32,
}

impl Registers {
    pub fn gpr_name(index: usize) -> &'static str {
        const NAMES: [&str; 32] = [
            "R0", "AT", "V0", "V1", "A0", "A1", "A2", "A3", "T0", "T1", "T2", "T3", "T4", "T5",
            "T6", "T7", "S0", "S1", "S2", "S3", "S4", "S5", "S6", "S7", "T8", "T9", "K0", "K1",
            "GP", "SP", "S8", "RA",
        ];

        NAMES.get(index).copied().unwrap_or("?") // TODO copied?
    }

    pub fn fpr_name(index: usize) -> &'static str {
        const NAMES: [&str; 32] = [
            "F0", "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12", "F13",
            "F14", "F15", "F16", "F17", "F18", "F19", "F20", "F21", "F22", "F23", "F24", "F25",
            "F26", "F27", "F28", "F29", "F30", "F31",
        ];

        NAMES.get(index).copied().unwrap_or("?") // TODO copied?
    }
}
