use arbitrary_int::u5;

#[derive(Clone, Copy, Debug, strum::Display, strum::EnumIter)]
#[repr(u8)]
pub enum Register {
    R0,
    AT,
    V0,
    V1,
    A0,
    A1,
    A2,
    A3,
    T0,
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    T7,
    S0,
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    T8,
    T9,
    K0,
    K1,
    GP,
    SP,
    FP,
    RA,
}

impl Register {
    pub fn index(&self) -> usize {
        *self as usize
    }
}

impl From<Register> for u5 {
    fn from(reg: Register) -> Self {
        u5::from_u8(reg as u8)
    }
}
