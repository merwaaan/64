//! Audio interface
//!
//! Manages DMA transfers of audio samples between RAM and the audio renderer.
//! The transferred audio data is 16-bit stereo, so 4 bytes per sample.
//!
//! https://n64brew.dev/wiki/Audio_Interface

use arbitrary_int::prelude::*;
use bitbybit::bitfield;

use crate::mapped_registers;

pub const START: u32 = 0x0450_0000;
pub const END: u32 = 0x0460_0000;

pub const REGISTERS_MASK: u32 = 0x1F;

/// RAM address for the next DMA transfer.
///
/// On hardware, this is write-only: reading it returns the DmaLength value.
#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct DmaRamAddress {
    #[bits(0..=23, rw)]
    value: u24,
}

// TODO move to regs macro
impl DmaRamAddress {
    pub fn write_masked(&mut self, value: u32) {
        *self = Self::new_with_raw_value(value & 0x00FF_FFF8);
    }
}

/// Length of the next DMA transfer.
/// Writing to this register starts a DMA transfer from RAM.
#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
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
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Control {
    #[bit(0, rw)]
    dma_enabled: bool,
}

impl Control {
    pub fn write_masked(&mut self, value: u32) {
        *self = Self::new_with_raw_value(value & 0x0000_0001);
    }
}

//0x0110_0000 prev
// 0x01F8_0000 with wc
// TODO wc baked in default, remove?
#[bitfield(u32, forbid_overlaps, instrospect, default = 0x01F8_0000, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
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
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
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
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct BitRate {
    #[bits(0..=3, rw)]
    value: u4,
}

impl BitRate {
    pub fn write_masked(&mut self, value: u32) {
        *self = Self::new_with_raw_value(value & 0x0000_000F);
    }
}

mapped_registers!(
    START,
    dma_ram_address: DmaRamAddress,
    dma_length: DmaLength,
    control: Control,
    status: Status,
    dac_rate: DacRate,
    bit_rate: BitRate,
);
