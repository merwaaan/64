use strum::{Display, EnumIter};

use crate::{
    data::Value,
    events::{EventType, Events},
    map::Location,
    mi::Interrupt,
    system::System,
};

const START: u32 = 0x0440_0000;
const END: u32 = 0x0450_0000;

pub type ViLocation = Location<START, END>;

const MASK: u32 = 0x3F;

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

const STATUS_REG: usize = 0;
const STATUS_LO: u32 = (STATUS_REG as u32) << 2;

// TODO flags

const FRAMEBUFFER_ADDR_REG: usize = 1; // "ORIGIN" in some docs
const FRAMEBUFFER_ADDR_LO: u32 = (FRAMEBUFFER_ADDR_REG as u32) << 2;
const FRAMEBUFFER_ADDR_MASK: u32 = 0x00FF_FFFF;

const WIDTH_REG: usize = 2;
const WIDTH_LO: u32 = (WIDTH_REG as u32) << 2;
const WIDTH_MASK: u32 = 0x0FFF;

const INTERRUPT_SCANLINE_REG: usize = 3;
const INTERRUPT_SCANLINE_LO: u32 = (INTERRUPT_SCANLINE_REG as u32) << 2;
const INTERRUPT_SCANLINE_MASK: u32 = 0x03FF;

const CURRENT_SCANLINE_REG: usize = 4;
//const CURRENT_SCANLINE_LO: u32 = (CURRENT_SCANLINE_REG as u32) << 2;

const BURST_REG: usize = 5;

const V_SYNC_REG: usize = 6; // TODO rn V_TOTAL?
const V_SYNC_MASK: u32 = 0x03FF;

const H_SYNC_REG: usize = 7;
const H_SYNC_LEAP_REG: usize = 8;
const H_VIDEO_REG: usize = 9;
const V_VIDEO_REG: usize = 10;
const V_BURST_REG: usize = 11;
const X_SCALE_REG: usize = 12;
const Y_SCALE_REG: usize = 13;

// NTSC 59.94 Hz, 262.5 scanlines
// PAL 50.00 Hz, 312.5 scanlines

#[derive(Debug, Clone, Copy)]
pub struct Vi {
    pub regs: [u32; 14],
}

impl Default for Vi {
    fn default() -> Self {
        Self {
            regs: [
                0, 0, 0, 0, 0, 0, 0, //0x271, // TODO PAL = 20D?
                0, 0, 0, 0, 0, 0, 0,
            ],
        }
    }
}

impl Vi {
    pub fn read<T: Value>(&self, addr: ViLocation) -> T {
        let reg = ((addr.relative() & MASK) >> 2) as usize;

        match reg {
            FRAMEBUFFER_ADDR_REG => {
                // TODO mask addr value???

                T::read_reg(&self.regs, addr.relative() & MASK)
            }
            CURRENT_SCANLINE_REG => {
                // TODO half scanlines???

                T::read_reg(&self.regs, addr.relative() & MASK)
            }
            _ => unimplemented!("Read VI register @ {:08X}", addr.relative()),
        }
    }

    pub fn write<T: Value>(s: &mut System, addr: ViLocation, data: T) {
        let reg = ((addr.relative() & MASK) >> 2) as usize;

        // TODO mask on w or r?

        match reg {
            STATUS_REG => {
                // TODO

                data.write_reg(&mut s.map.vi.regs, addr.relative() & MASK);
            }

            FRAMEBUFFER_ADDR_REG => {
                data.write_reg(&mut s.map.vi.regs, addr.relative() & MASK);

                s.map.vi.regs[FRAMEBUFFER_ADDR_REG] &= FRAMEBUFFER_ADDR_MASK;
            }

            WIDTH_REG => {
                data.write_reg(&mut s.map.vi.regs, addr.relative() & MASK);

                s.map.vi.regs[WIDTH_REG] &= WIDTH_MASK;
            }

            INTERRUPT_SCANLINE_REG => {
                data.write_reg(&mut s.map.vi.regs, addr.relative() & MASK);

                s.map.vi.regs[INTERRUPT_SCANLINE_REG] &= INTERRUPT_SCANLINE_MASK;
            }

            CURRENT_SCANLINE_REG => {
                // Writing anything to this register clears the interrupt

                s.map.mi.clear_pending_interrupt(Interrupt::Vi, &mut s.cop0);
            }

            BURST_REG => {
                data.write_reg(&mut s.map.vi.regs, addr.relative() & MASK);
            }

            V_SYNC_REG => {
                // TODO

                data.write_reg(&mut s.map.vi.regs, addr.relative() & MASK);

                s.map.vi.regs[V_SYNC_REG] &= V_SYNC_MASK;
            }

            H_SYNC_REG => {
                // TODO

                data.write_reg(&mut s.map.vi.regs, addr.relative() & MASK);
            }

            H_SYNC_LEAP_REG => {
                // TODO

                data.write_reg(&mut s.map.vi.regs, addr.relative() & MASK);
            }

            H_VIDEO_REG => {
                // TODO

                data.write_reg(&mut s.map.vi.regs, addr.relative() & MASK);
            }

            V_VIDEO_REG => {
                // TODO

                data.write_reg(&mut s.map.vi.regs, addr.relative() & MASK);
            }

            V_BURST_REG => {
                // TODO

                data.write_reg(&mut s.map.vi.regs, addr.relative() & MASK);
            }

            X_SCALE_REG => {
                // TODO

                data.write_reg(&mut s.map.vi.regs, addr.relative() & MASK);
            }

            Y_SCALE_REG => {
                // TODO

                data.write_reg(&mut s.map.vi.regs, addr.relative() & MASK);
            }

            _ => unimplemented!("Write VI register {:X} @ {:08X}", data, addr.relative()),
        }
    }

    pub fn reg_info(addr: ViLocation) -> Option<&'static str> {
        match addr.relative() & MASK {
            STATUS_LO => Some("VI_STATUS"),
            FRAMEBUFFER_ADDR_LO => Some("VI_FRAMEBUFFER_ADDR"),
            WIDTH_LO => Some("VI_WIDTH"),
            INTERRUPT_SCANLINE_LO => Some("VI_INTERRUPT_SCANLINE"),
            _ => None,
        }
    }

    pub fn scanline_completed(s: &mut System) {
        // Update the status register

        // s.map.pi.regs[STATUS_REG] |= STATUS_DMA_COMPLETED_MASK;
        // s.map.pi.regs[STATUS_REG] &= !STATUS_DMA_BUSY_MASK;
        // TODO IO busy?

        // Raise the interrupt
        // TODO >= or ==???

        if s.map.vi.regs[CURRENT_SCANLINE_REG] == s.map.vi.regs[V_SYNC_REG] {
            s.map.vi.regs[CURRENT_SCANLINE_REG] = 0;
        }

        s.map.vi.regs[CURRENT_SCANLINE_REG] =
            s.map.vi.regs[CURRENT_SCANLINE_REG].wrapping_add(1) & 0x3FF;

        if s.map.vi.regs[CURRENT_SCANLINE_REG] == s.map.vi.regs[INTERRUPT_SCANLINE_REG] {
            s.map.mi.set_pending_interrupt(Interrupt::Vi, &mut s.cop0);
        }

        // Schedule the next scanline

        Events::push(s, EventType::ViScanlineComplete, /*1587*/ 10000); // TODO
    }

    pub fn address_info(addr: ViLocation) -> Option<&'static str> {
        // TODO check masks!
        // TODO normalize strings

        match addr.relative() & MASK {
            STATUS_LO => Some("VI_STATUS"),
            FRAMEBUFFER_ADDR_LO => Some("VI_FRAMEBUFFER_ADDR"),
            WIDTH_LO => Some("VI_WIDTH"),
            INTERRUPT_SCANLINE_LO => Some("VI_INTERRUPT_SCANLINE"),
            // TODO others
            _ => None,
        }
    }

    pub(crate) fn color32(&self) -> bool {
        self.regs[STATUS_REG] & 0b11 == 0b11
        // TODO other modes?
    }

    pub(crate) fn framebuffer_address(&self) -> u32 {
        self.regs[FRAMEBUFFER_ADDR_REG]
    }

    pub fn framebuffer_width(&self) -> usize {
        self.regs[WIDTH_REG] as usize
    }

    pub fn framebuffer_height(&self) -> usize {
        480 // TODOself.regs[V_SYNC_REG] as usize
    }

    pub fn extract_framebuffer(s: &System) -> (Vec<u8>, usize, usize) {
        let base_addr = s.map.vi.framebuffer_address();
        let width = s.map.vi.framebuffer_width();
        let height = s.map.vi.framebuffer_height();

        let mut data = Vec::with_capacity(width * height * 4);

        if s.map.vi.color32() {
            for y in 0..height {
                for x in 0..width {
                    let pixel = s.read::<u32>(base_addr + ((y * width + x) * 4) as u32);

                    data.push((pixel >> 24) as u8);
                    data.push((pixel >> 16) as u8);
                    data.push((pixel >> 8) as u8);
                    data.push(0xFF); // TODO real val
                }
            }
        } else {
            for y in 0..height {
                for x in 0..width {
                    let pixel = s.read::<u16>(base_addr + ((y * width + x) * 2) as u32);

                    data.push(Self::b5_to_b8(pixel >> 11));
                    data.push(Self::b5_to_b8(pixel >> 6));
                    data.push(Self::b5_to_b8(pixel >> 1));
                    data.push(0xFF); // TODO real val
                }
            }
        }

        (data, width, height)
    }

    fn b5_to_b8(value: u16) -> u8 {
        (((value & 0x1F) * 255) / 31) as u8
    }
}
