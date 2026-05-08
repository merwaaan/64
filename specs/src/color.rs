use arbitrary_int::prelude::*;
use bitbybit::bitfield;

/// 32-bit color format
#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
pub struct RGBA8888 {
    #[bits(24..=31, rw)]
    pub red: u8,

    #[bits(16..=23, rw)]
    pub green: u8,

    #[bits(8..=15, rw)]
    pub blue: u8,

    #[bits(0..=7, rw)]
    pub alpha: u8,
}

impl RGBA8888 {
    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::new_with_raw_value(
            ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32),
        )
    }
}

impl From<RGBA5551> for RGBA8888 {
    fn from(rgba5551: RGBA5551) -> Self {
        let r5 = ((rgba5551.raw_value() >> 11) & 0x1F) as u8;
        let g5 = ((rgba5551.raw_value() >> 6) & 0x1F) as u8;
        let b5 = ((rgba5551.raw_value() >> 1) & 0x1F) as u8;
        let a1 = (rgba5551.raw_value() & 0x1) as u8;

        // Replicate the upper bits into the lower bits

        Self::from_rgba(
            (r5 << 3) | (r5 >> 2),
            (g5 << 3) | (g5 >> 2),
            (b5 << 3) | (b5 >> 2),
            a1 * 255,
        )
    }
}

/// 16-bit color format
#[bitfield(u16, forbid_overlaps, instrospect, default = 0, debug)]
pub struct RGBA5551 {
    #[bits(11..=15, rw)]
    pub red: u5,

    #[bits(6..=10, rw)]
    pub green: u5,

    #[bits(1..=5, rw)]
    pub blue: u5,

    #[bit(0, rw)]
    pub alpha: bool,
}

impl RGBA5551 {
    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        // Round to nearest

        Self::new_with_raw_value(
            (((r as u16 * 31 + 127) / 255) << 11)
                | (((g as u16 * 31 + 127) / 255) << 6)
                | (((b as u16 * 31 + 127) / 255) << 1)
                | (a >= 128) as u16,
        )
    }
}
