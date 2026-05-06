//! Video interface
//!
//! TODO
//!
//! https://n64brew.dev/wiki/Video_Interface

use arbitrary_int::prelude::*;
use bitbybit::bitfield;

use crate::mapped_registers;

pub const START: u32 = 0x0440_0000;
pub const END: u32 = 0x0450_0000;

pub const REGISTERS_MASK: u32 = 0x3F; // TODO check + rename

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Control {
    #[bit(16, rw)]
    dither: bool,

    #[bits(12..=15, rw)] // TODO enum
    pixel_advance: u4,

    #[bit(11, rw)]
    kill_writes: bool,

    #[bits(8..=9, rw)] // TODO enum
    antialias_mode: u2,

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

    #[bits(0..=1, rw)] // TODO enum
    color_mode: u2,
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

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct VerticalTotal {
    #[bits(0..=9, rw)]
    value: u10,
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct HorizontalTotal {
    #[bits(16..=20, rw)]
    leap: u5,

    #[bits(0..=11, rw)]
    total: u12,
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct HorizontalTotalLeap {
    #[bits(16..=27, rw)]
    leap_a: u12,

    #[bits(0..=11, rw)]
    leap_b: u12,
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct HorizontalVideo {
    #[bits(16..=25, rw)]
    start: u10,

    #[bits(0..=9, rw)]
    end: u10,
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct VerticalVideo {
    #[bits(16..=25, rw)]
    start: u10,

    #[bits(0..=9, rw)]
    end: u10,
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct VerticalBurst {
    #[bits(16..=25, rw)]
    start: u10,

    #[bits(0..=9, rw)]
    end: u10,
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct HorizontalScale {
    #[bits(16..=27, rw)]
    offset: u12,

    #[bits(0..=11, rw)]
    scale: u12,
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct VerticalScale {
    #[bits(16..=25, rw)]
    offset: u10,

    #[bits(0..=11, rw)]
    scale: u12,
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

// TODO test addr + staged data??
