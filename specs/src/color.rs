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

// impl From<RGBA8888> for RGBA5551 {
//     fn from(rgba: RGBA8888) -> Self {
//         Self::default()
//             .with_red(rgba.red())
//             .with_green(rgba.green())
//             .with_blue(rgba.blue())
//             .with_alpha(rgba.alpha())
//     }
// }
