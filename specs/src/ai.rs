//! Audio interface
//!
//! Manages DMA transfers of audio samples between RAM and the audio renderer.
//! The transferred audio data is 16-bit stereo, so 4 bytes per sample.
//!
//! https://n64brew.dev/wiki/Audio_Interface

use arbitrary_int::prelude::*;
use bitbybit::bitfield;

pub const START: u32 = 0x0450_0000;
pub const END: u32 = 0x0460_0000;

// TODO wo
#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
pub struct DmaRamAddress {
    #[bits(0..=23, rw)]
    value: u24,
}

impl DmaRamAddress {
    pub fn write_masked(&mut self, value: u32) {
        *self = Self::new_with_raw_value(value & 0x00FF_FFF8);
    }
}

// TODO wo
#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
pub struct DmaLength {
    #[bits(0..=17, rw)]
    value: u18,
}

impl DmaLength {
    pub fn write_masked(&mut self, value: u32) {
        *self = Self::new_with_raw_value(value & 0x0003_FFF8);
    }
}

// TODO wo
// TODO masked or not? test
#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
pub struct Control {
    #[bit(0, rw)]
    dma_enabled: bool,
}

impl Control {
    pub fn write_masked(&mut self, value: u32) {
        *self = Self::new_with_raw_value(value & 0x0000_0001);
    }
}

// TODO r? w?
#[bitfield(u32, forbid_overlaps, instrospect, default = 0x0110_0000, debug)]
pub struct Status {
    #[bit(31, rw)]
    dma_full: bool,

    #[bit(30, rw)]
    dma_busy: bool,

    #[bit(25, rw)]
    dma_enabled: bool,

    #[bit(19, rw)]
    word_clock: bool,

    #[bit(16, rw)]
    bit_clock: bool,

    #[bits(1..=14, rw)]
    count: u14,

    #[bit(0, rw)]
    dma_full_mirror: bool,
}

// TODO wo
#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
pub struct DacRate {
    #[bits(0..=13, rw)]
    value: u14,
}

impl DacRate {
    pub fn write_masked(&mut self, value: u32) {
        *self = Self::new_with_raw_value(value & 0x0003_FFFF);
    }
}

// TODO wo
#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
pub struct BitRate {
    #[bits(0..=3, rw)]
    value: u4,
}

impl BitRate {
    pub fn write_masked(&mut self, value: u32) {
        *self = Self::new_with_raw_value(value & 0x0000_000F);
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Registers {
    pub dma_ram_address: DmaRamAddress,
    pub dma_length: DmaLength,
    pub control: Control,
    pub status: Status,
    pub dac_rate: DacRate,
    pub bit_rate: BitRate,
}
