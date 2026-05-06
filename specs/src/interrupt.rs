// TODO doc
#[derive(Debug, Clone, Copy, strum::Display, strum::EnumIter)]
#[repr(u8)]
pub enum Interrupt {
    Sp = 1,
    Si = 1 << 1,
    Ai = 1 << 2,
    Vi = 1 << 3,
    Pi = 1 << 4,
    Dp = 1 << 5,
}
