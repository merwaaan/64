// The MIPS registers are 64-bits.
//
// The Nintendo 64 runs in 32-bit mode and games mostly manipulate 32-bit values.
// However, some instructions rely on the full 64-bit range of the registers (eg. DSLL, DSRL).

// TODO isn't this just data<u64>?
#[derive(Default, Copy, Clone)]
pub struct Reg64(u64);

impl Reg64 {
    pub fn get(&self) -> u32 {
        self.0 as u32
    }

    pub fn set(&mut self, value: u32) {
        // Sign-extend the 32-bit value
        self.0 = value as i32 as u64;
    }

    pub fn get64(&self) -> u64 {
        self.0
    }

    pub fn set64(&mut self, value: u64) {
        self.0 = value;
    }
}

// TODO optim: avoid branch in get?

#[derive(Clone, Copy)]
pub enum GPReg {
    // r0 always reads as 0 and cannot be written to
    Zero,
    N(Reg64),
}

// TODO use Data trait?
impl GPReg {
    pub fn get(&self) -> u32 {
        match self {
            GPReg::Zero => 0,
            GPReg::N(r) => r.get(),
        }
    }

    pub fn set(&mut self, value: u32) {
        match self {
            GPReg::Zero => {}
            GPReg::N(r) => r.set(value),
        }
    }

    pub fn get64(&self) -> u64 {
        match self {
            GPReg::Zero => 0,
            GPReg::N(r) => r.get64(),
        }
    }

    pub fn set64(&mut self, value: u64) {
        match self {
            GPReg::Zero => {}
            GPReg::N(r) => r.set64(value),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Registers {
    pub pc: u32,

    pub gpr: [GPReg; 32],

    // TODO move out?
    pub fpr: [Reg64; 32],
    pub fcr: u32,

    pub mult_hi: Reg64,
    pub mult_lo: Reg64,

    pub load_linked_bit: bool,
    pub load_linked_addr: u32,
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}

impl Registers {
    pub fn new() -> Self {
        Self {
            pc: 0,

            gpr: std::array::from_fn(|i| {
                if i == 0 {
                    GPReg::Zero
                } else {
                    GPReg::N(Reg64::default())
                }
            }),

            fpr: [Reg64::default(); 32],
            fcr: 0,

            mult_hi: Reg64::default(),
            mult_lo: Reg64::default(),

            load_linked_bit: false,
            load_linked_addr: 0,
        }
    }

    pub fn f_rounding_mode(&self) -> RoundingMode {
        match self.fcr & 0x3 {
            0 => RoundingMode::Nearest,
            1 => RoundingMode::Zero,
            2 => RoundingMode::Infinity,
            _ => RoundingMode::NegativeInfinity,
        }
    }

    pub fn f_64(&self) -> bool {
        self.fcr & 0x40 != 0
    }
}

#[derive(Debug)]
#[repr(u32)]
pub enum RoundingMode {
    Nearest,
    Zero,
    Infinity,
    NegativeInfinity,
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
