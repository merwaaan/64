/// Evaluates if a register that is part of a struct made of contiguous registers overlaps a range.
///
/// Typically used to check if a write touches a specific register to trigger side effects.
#[macro_export]
macro_rules! register_overlaps {
    ($start:expr, $end:expr, $registers_struct:ident :: $register_field:ident) => {{
        const REG_OFFSET: u32 = std::mem::offset_of!($registers_struct, $register_field) as u32;

        $start < (REG_OFFSET + 4) && $end > REG_OFFSET
    }};
}

pub trait BitTest {
    /// Checks if a bit is set.
    fn bit_is_set<const BIT: u32>(self) -> bool;
}

impl<T> BitTest for T
where
    T: std::ops::Shr<u32, Output = T> + std::ops::BitAnd<Output = T> + From<u8> + PartialEq,
{
    #[inline(always)]
    fn bit_is_set<const BIT: u32>(self) -> bool {
        (self >> BIT) & T::from(1) == T::from(1)
    }
}
