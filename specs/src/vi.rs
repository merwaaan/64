//! Video interface
//!
//! TODO
//!
//! https://n64brew.dev/wiki/Video_Interface

use arbitrary_int::prelude::*;
use bitbybit::{bitenum, bitfield};

use crate::mapped_registers;

pub const START: u32 = 0x0440_0000;
pub const END: u32 = 0x0450_0000;

pub const REGISTERS_MASK: u32 = 0x3F; // TODO check + rename

#[bitenum(u2, exhaustive = true)]
#[derive(PartialEq, Debug)]
pub enum ColorMode {
    // 32-bit color
    Rgba8888 = 0b11,
    // 16-bit color
    Rgba5551 = 0b10,
    // Reserved
    Reserved = 0b01,
    // No display
    Off = 0b00,
}

#[bitenum(u2, exhaustive = true)]
#[derive(Debug)]
pub enum AntiAliasingMode {
    Replicate = 0b11,
    Resample = 0b10,
    AntiAliasNeeded = 0b01,
    AntiAliasAlways = 0b00,
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Control {
    #[bit(16, rw)]
    dither: bool,

    #[bits(12..=15, rw)]
    pixel_advance: u4,

    #[bit(11, rw)]
    kill_writes: bool,

    /// Unused bit, still writable according to hardwaretests
    #[bit(10, rw)]
    unused: bool,

    #[bits(8..=9, rw)]
    antialias_mode: AntiAliasingMode,

    #[bit(7, rw)]
    test_mode: bool,

    #[bit(6, rw)]
    serrate: bool,

    #[bit(5, rw)]
    vbus_clock: bool,

    #[bit(4, rw)]
    divot: bool,

    #[bit(3, rw)]
    gamma: bool,

    #[bit(2, rw)]
    gamma_dither: bool,

    #[bits(0..=1, rw)]
    color_mode: ColorMode,
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Origin {
    #[bits(0..=23, rw)]
    ram_address: u24,
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Width {
    #[bits(0..=11, rw)]
    value: u12,
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct InterruptLine {
    #[bits(0..=9, rw)]
    value: u10,
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct CurrentLine {
    #[bits(1..=9, rw)]
    line: u9,

    #[bit(0, rw)]
    field: u1,
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Burst {
    #[bits(20..=29, rw)]
    vburst_start: u10,

    #[bits(16..=19, rw)]
    vsync_height: u4,

    #[bits(8..=15, rw)]
    burst_width: u8,

    #[bits(0..=7, rw)]
    hsync_width: u8,
}

pub const BURST_NTSC: u32 = 0x03E5_2239;
pub const BURST_PAL: u32 = 0x0404_233A;

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct VerticalTotal {
    #[bits(0..=9, rw)]
    value: u10,
}

pub const VERTICAL_TOTAL_NTSC_PROGRESSIVE: u32 = 525;
pub const VERTICAL_TOTAL_NTSC_INTERLACED: u32 = 524;
pub const VERTICAL_TOTAL_PAL_PROGRESSIVE: u32 = 625;
pub const VERTICAL_TOTAL_PAL_INTERLACED: u32 = 624;

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct HorizontalTotal {
    #[bits(16..=20, rw)]
    leap: u5,

    #[bits(0..=11, rw)]
    total: u12,
}

const LEAP_NTSC: u32 = 0x15;
const LEAP_PAL: u32 = 0;

pub const HORIZONTAL_TOTAL_NTSC: u32 = (LEAP_NTSC << 16) | 3093;
pub const HORIZONTAL_TOTAL_PAL: u32 = (LEAP_PAL << 16) | 3177;
pub const HORIZONTAL_TOTAL_MPAL_PROGRESSIVE: u32 = (LEAP_PAL << 16) | 3089;
pub const HORIZONTAL_TOTAL_MPAL_INTERLACED: u32 = (LEAP_PAL << 16) | 3088;

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct HorizontalTotalLeap {
    #[bits(16..=27, rw)]
    leap_a: u12,

    #[bits(0..=11, rw)]
    leap_b: u12,
}

pub const HORIZONTAL_TOTAL_LEAP_NTSC: u32 = 0x0C6E_0C6F; // A = 3182, B = 3183
pub const HORIZONTAL_TOTAL_LEAP_PAL: u32 = 0; // TODO find out

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct HorizontalVideo {
    #[bits(16..=25, rw)]
    start: u10,

    #[bits(0..=9, rw)]
    end: u10,
}

pub const HORIZONTAL_VIDEO_NTSC: u32 = 0x006C_02EC; // start = 108, end = 128
pub const HORIZONTAL_VIDEO_PAL: u32 = 0x0080_0300; // start = 128, end = 768

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct VerticalVideo {
    #[bits(16..=25, rw)]
    start: u10,

    #[bits(0..=9, rw)]
    end: u10,
}

pub const VERTICAL_VIDEO_NTSC: u32 = 0x0025_01FF; // start = 37, end = 511
pub const VERTICAL_VIDEO_PAL: u32 = 0x005F_0239; // start = 95, end = 569

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct VerticalBurst {
    #[bits(16..=25, rw)]
    start: u10,

    #[bits(0..=9, rw)]
    end: u10,
}

pub const VERTICAL_BURST_NTSC: u32 = 0x000E_0204; // start = 14, end = 516
pub const VERTICAL_BURST_PAL: u32 = 0x0009_026B; // start = 9, end = 619

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct HorizontalScale {
    /// Subpixel offset, 2.10 format
    #[bits(16..=27, rw)]
    offset: u12,

    /// 1 / scale factor, 2.10 format
    #[bits(0..=11, rw)]
    scale: u12,
}

const WIDTH_NTSC: u32 = 640;
const HEIGHT_NTSC: u32 = 240; // TODO interlaced?
// const WIDTH_PAL: u32 = 720;
// const HEIGHT_PAL: u32 = 288; // TODO interlaced?

// TODO handle PAL
pub fn horizontal_scale_from_width(width: u32) -> HorizontalScale {
    let scale = ((width as f32) / WIDTH_NTSC as f32 * 1024.0) as u16 & 0xFFF;

    // (1024 = 1 in 2.10 format so no stretching)

    HorizontalScale::default().with_scale(u12::new(scale))
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct VerticalScale {
    /// Subpixel offset, 0.10 format
    #[bits(16..=25, rw)]
    offset: u10,

    /// 1 / scale factor, 2.10 format
    #[bits(0..=11, rw)]
    scale: u12,
}

// TODO handle PAL
// TODO interlaced?
pub fn vertical_scale_from_height(height: u32) -> VerticalScale {
    let scale = ((height as f32) / HEIGHT_NTSC as f32 * 1024.0) as u16 & 0xFFF;

    VerticalScale::default().with_scale(u12::new(scale))
}

mapped_registers!(
    START,
    control: Control,
    origin: Origin,
    width: Width,
    interrupt_line: InterruptLine,
    current_line: CurrentLine,
    burst: Burst,
    vertical_total: VerticalTotal,
    horizontal_total: HorizontalTotal,
    horizontal_leap: HorizontalTotalLeap,
    horizontal_video: HorizontalVideo,
    vertical_video: VerticalVideo,
    vertical_burst: VerticalBurst,
    horizontal_scale: HorizontalScale,
    vertical_scale: VerticalScale,
);

// TODO test addr + staged data?

// TODO helper to configure all the regs with size, origin, standard, mode
