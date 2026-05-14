// TODO doc
#[derive(Debug, Clone, Copy, strum::Display, strum::EnumIter, strum::EnumCount)]
#[repr(u8)]
pub enum Interrupt {
    Sp = 1,
    Si = 1 << 1,
    Ai = 1 << 2,
    Vi = 1 << 3,
    Pi = 1 << 4,
    Dp = 1 << 5,
}

impl Interrupt {
    /// Returns the mask to clear the interrupt via the MI Mask register.
    pub fn clear_mask(self) -> u32 {
        1 << ((self as u8).trailing_zeros() * 2)
    }

    /// Returns the mask to set the interrupt via the MI Mask register.
    pub fn set_mask(self) -> u32 {
        1 << ((self as u8).trailing_zeros() * 2 + 1)
    }
}
