use arbitrary_int::prelude::*;
use bitbybit::bitfield;
use strum::{Display, EnumIter};

use crate::{
    cpu,
    events::{EventType, Events},
    location::Location,
    mi::Interrupt,
    register_overlaps,
    system::{Address, System},
    value::Value,
};

pub type ViLocation = Location<0x0440_0000, 0x0450_0000>;

#[derive(Debug, Display, Clone, Copy, EnumIter)]
#[repr(u32)]
pub enum Register {
    Status,
    FramebufferAddr,
    Width,
    InterruptScanline,
    CurrentScanline,
    Burst,
    VSync,
    HSync,
    HSyncLeap,
    /// Horizontal start/end
    HVideo,
    /// Vertical start/end
    VVideo,
    VBurst,
    XScale,
    YScale,
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Control {
    #[bits(0..=1, rw)] // TODO enum
    color_mode: u2,

    #[bit(2, rw)]
    gamma_dither: bool,

    #[bit(3, rw)]
    gamma: bool,

    #[bit(4, rw)]
    divot: bool,

    #[bit(5, rw)]
    vbus_clock: bool,

    #[bit(6, rw)]
    serrate: bool,

    #[bit(7, rw)]
    test_mode: bool,

    #[bits(8..=9, rw)] // TODO enum
    antialias_mode: u2,

    #[bit(11, rw)]
    kill_writes: bool,

    #[bits(12..=15, rw)] // TODO enum
    pixel_advance: u4,

    #[bit(16, rw)]
    dither: bool,
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
    #[bit(0, rw)]
    field: u1,

    #[bits(1..=9, rw)]
    line: u9,
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Burst {
    #[bits(0..=7, rw)]
    hsync_width: u8,

    #[bits(8..=15, rw)]
    burst_width: u8,

    #[bits(16..=19, rw)]
    vsync_height: u4,

    #[bits(20..=29, rw)]
    vburst_start: u10,
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
    #[bits(0..=11, rw)]
    total: u12,

    #[bits(16..=20, rw)]
    leap: u5,
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct HorizontalTotalLeap {
    #[bits(0..=11, rw)]
    leap_b: u12,

    #[bits(16..=27, rw)]
    leap_a: u12,
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct StartEnd {
    #[bits(0..=9, rw)]
    end: u10,

    #[bits(16..=25, rw)]
    start: u10,
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct HorizontalScale {
    #[bits(0..=11, rw)]
    scale: u12,

    #[bits(16..=27, rw)]
    offset: u12,
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct VerticalScale {
    #[bits(0..=11, rw)]
    scale: u12,

    #[bits(16..=25, rw)]
    offset: u10,
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Registers {
    pub control: Control,
    pub origin: Origin,
    pub width: Width,
    pub interrupt_line: InterruptLine,
    pub current_line: CurrentLine,
    pub burst: Burst,
    pub vertical_total: VerticalTotal,
    pub horizontal_total: HorizontalTotal,
    pub horizontal_leap: HorizontalTotalLeap,
    pub horizontal_video: StartEnd,
    pub vertical_video: StartEnd,
    pub vertical_burst: StartEnd,
    pub x_scale: HorizontalScale,
    pub y_scale: VerticalScale,
    // TODO others?
    test_address: u32,
    staged_data: u32,
}

impl Registers {
    pub fn read<T: Value>(&self, offset: u32) -> T {
        let words = bytemuck::cast_slice(bytemuck::bytes_of(self));

        T::read_reg(words, offset)
    }

    pub fn write<T: Value>(&mut self, offset: u32, data: T) {
        let mut words = bytemuck::cast_slice_mut(bytemuck::bytes_of_mut(self));

        data.write_reg(&mut words, offset);
    }
}

const REGISTERS_MASK: u32 = 0x3F;

// NTSC 59.94 Hz, 262.5 scanlines
// PAL 50.00 Hz, 312.5 scanlines
const NTSC_FREQUENCY: f64 = 60.0; // TODO exact?
const _PAL_FREQUENCY: f64 = 50.0;

const TOTAL_SCANLINES: usize = 525; // TODO depends????
const FRAME_CPU_CYCLES: usize = (cpu::FREQUENCY / NTSC_FREQUENCY) as usize;
pub const SCANLINE_CPU_CYCLES: usize = FRAME_CPU_CYCLES / TOTAL_SCANLINES; // TODO fractional part?

#[derive(Debug, Clone, Copy)]
pub struct Vi {
    regs: Registers,
}

impl Default for Vi {
    fn default() -> Self {
        Self {
            regs: Registers::default(),
        }
    }
}

impl Vi {
    pub fn regs(&self) -> &Registers {
        &self.regs
    }

    pub fn read<T: Value>(s: &System, addr: ViLocation) -> T {
        s.vi.regs.read(addr.relative() & REGISTERS_MASK)
    }

    pub fn write<T: Value>(s: &mut System, addr: ViLocation, data: T) {
        let current_line = s.vi.regs.current_line;

        let offset = addr.relative() & REGISTERS_MASK;

        s.vi.regs.write(offset, data);

        if register_overlaps!(offset, offset + T::BYTES as u32, Registers::current_line) {
            s.mi.clear_pending_interrupt(Interrupt::Vi, &mut s.cop0);

            // CURRENT_LINE is read-only
            s.vi.regs.current_line = current_line;
        }
    }

    pub(crate) fn framebuffer_address(&self) -> u32 {
        self.regs.origin.ram_address().value()
    }

    pub fn framebuffer_width(&self) -> usize {
        self.regs.width.value().value() as usize
    }

    pub fn framebuffer_height(&self) -> usize {
        480 // TODOself.regs[V_SYNC_REG] as usize
    }

    pub fn scanline_completed(s: &mut System) {
        // Increment the current scanline by 2 half scanlines
        // TODO Toggle the field bit?

        s.vi.regs
            .current_line
            .set_line(s.vi.regs.current_line.line().wrapping_add(u9::new(1))); // TODO halfline overlap in struct?
        //s.vi.regs[CURRENT_SCANLINE_REG] = s.vi.regs[CURRENT_SCANLINE_REG].wrapping_add(2) & 0x3FF;

        // Reset the current scanline to 0 if it matches the V_SYNC register

        if s.vi.regs.current_line.line().value() >= s.vi.regs.vertical_total.value().value() {
            s.vi.regs.current_line.set_line(u9::ZERO); // TODO halfline overlap?
        }

        // Raise an interrupt if the current scanline matches the interrupt scanline
        // TODO >= or ==???

        if s.vi.regs.current_line.line().value() == s.vi.regs.interrupt_line.value().value() {
            // TODO halfline overlap?
            s.mi.set_pending_interrupt(Interrupt::Vi, &mut s.cop0);
        }

        // Schedule the next scanline
        // probably needd to be computed dynamically based on the current height?

        Events::push(s, EventType::ViScanlineComplete, SCANLINE_CPU_CYCLES);
    }

    pub fn extract_framebuffer(s: &mut System) -> (Vec<u8>, usize, usize) {
        let base_addr = s.vi.framebuffer_address();
        let width = s.vi.framebuffer_width();
        let height = s.vi.framebuffer_height();

        let mut data = Vec::with_capacity(width * height * 4);

        let color32 = s.vi.regs.control.color_mode().value() == 3;

        if color32 {
            for y in 0..height {
                for x in 0..width {
                    // TODO optim: directly access RAM with read_block
                    let pixel = s
                        .read::<u32>(Address::p(base_addr + ((y * width + x) * 4) as u32))
                        .expect("Invalid pixel address");

                    data.push((pixel >> 24) as u8);
                    data.push((pixel >> 16) as u8);
                    data.push((pixel >> 8) as u8);
                    data.push(0xFF); // TODO real val
                }
            }
        } else {
            for y in 0..height {
                for x in 0..width {
                    let pixel = s
                        .read::<u16>(Address::p(base_addr + ((y * width + x) * 2) as u32))
                        .expect("Invalid pixel address");

                    data.push(Self::b5_to_b8(pixel >> 11));
                    data.push(Self::b5_to_b8(pixel >> 6));
                    data.push(Self::b5_to_b8(pixel >> 1));
                    data.push(0xFF); // TODO real val
                }
            }
        }

        (data, width, height)
    }

    // TODO move out, used elsewhere
    fn b5_to_b8(value: u16) -> u8 {
        (((value & 0x1F) * 255) / 31) as u8
    }
}
