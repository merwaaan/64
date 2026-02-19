use strum::{Display, EnumIter};

use crate::{
    data::Data,
    events::{Event, EventType},
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
    HVideo,
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
pub const WIDTH_MASK: u32 = 0x0FFF;

const INTERRUPT_SCANLINE_REG: usize = 3;
const INTERRUPT_SCANLINE_LO: u32 = (INTERRUPT_SCANLINE_REG as u32) << 2;
pub const INTERRUPT_SCANLINE_MASK: u32 = 0x03FF;

const CURRENT_SCANLINE_REG: usize = 4;
const CURRENT_SCANLINE_LO: u32 = (CURRENT_SCANLINE_REG as u32) << 2;

const BURST_REG: usize = 5;
const BURST_LO: u32 = (BURST_REG as u32) << 2;

const V_SYNC_REG: usize = 6;
const V_SYNC_LO: u32 = (V_SYNC_REG as u32) << 2;

const H_SYNC_REG: usize = 7;
const H_SYNC_LO: u32 = (H_SYNC_REG as u32) << 2;

const H_SYNC_LEAP_REG: usize = 8;
const H_SYNC_LEAP_LO: u32 = (H_SYNC_LEAP_REG as u32) << 2;

const H_VIDEO_REG: usize = 9;
const H_VIDEO_LO: u32 = (H_VIDEO_REG as u32) << 2;

const V_VIDEO_REG: usize = 10;
const V_VIDEO_LO: u32 = (V_VIDEO_REG as u32) << 2;

const V_BURST_REG: usize = 11;
const V_BURST_LO: u32 = (V_BURST_REG as u32) << 2;

const X_SCALE_REG: usize = 12;
const X_SCALE_LO: u32 = (X_SCALE_REG as u32) << 2;

const Y_SCALE_REG: usize = 13;
const Y_SCALE_LO: u32 = (Y_SCALE_REG as u32) << 2;

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
                0, 0, 0, 0, 0, 0, 0x271, // TODO PAL = 20D?
                0, 0, 0, 0, 0, 0, 0,
            ],
        }
    }
}

impl Vi {
    pub fn read<T: Data>(&self, addr: ViLocation) -> T {
        let reg = ((addr.relative() & MASK) >> 2) as usize;

        match reg {
            FRAMEBUFFER_ADDR_REG => T::from_u32(self.regs[FRAMEBUFFER_ADDR_REG]),

            // TODO half scanlines???
            CURRENT_SCANLINE_REG => T::from_u32(self.regs[CURRENT_SCANLINE_REG]),

            _ => unimplemented!("Read VI register @ {:08X}", addr.relative()),
        }
    }

    pub fn write<T: Data>(s: &mut System, addr: ViLocation, data: T) {
        let reg = ((addr.relative() & MASK) >> 2) as usize;

        let data = data.to_u32(); // TODO temp hack, should be able to write any size

        // TODO mask on w or r?

        match reg {
            STATUS_REG => {
                // TODO
                log::warn!("Write VI_STATUS {:X}", data.to_u32());

                s.map.vi.regs[STATUS_REG] = data;
            }

            FRAMEBUFFER_ADDR_REG => {
                s.map.vi.regs[FRAMEBUFFER_ADDR_REG] = data & FRAMEBUFFER_ADDR_MASK;
            }

            WIDTH_REG => {
                s.map.vi.regs[WIDTH_REG] = data & WIDTH_MASK;
            }

            INTERRUPT_SCANLINE_REG => {
                // TODO
                log::warn!("Write VI_INTERRUPT_SCANLINE {:X}", data.to_u32());

                s.map.vi.regs[INTERRUPT_SCANLINE_REG] = data & INTERRUPT_SCANLINE_MASK;
            }

            CURRENT_SCANLINE_REG => {
                // Writing anything to this register clears the Interrupt and resets the current scanline

                s.map.mi.clear_pending_interrupt(Interrupt::Vi);

                s.map.vi.regs[CURRENT_SCANLINE_REG] = 0; // TODO really?
            }

            BURST_REG => {
                s.map.vi.regs[BURST_REG] = data;
            }

            V_SYNC_REG => {
                // TODO
                log::warn!("Write VI_V_SYNC {:X}", data.to_u32());

                s.map.vi.regs[V_SYNC_REG] = data;
            }

            H_SYNC_REG => {
                // TODO
                log::warn!("Write VI_H_SYNC {:X}", data.to_u32());

                s.map.vi.regs[H_SYNC_REG] = data;
            }

            H_SYNC_LEAP_REG => {
                // TODO
                log::warn!("Write VI_H_SYNC_LEAP {:X}", data.to_u32());

                s.map.vi.regs[H_SYNC_LEAP_REG] = data;
            }

            H_VIDEO_REG => {
                // TODO
                log::warn!("Write VI_H_VIDEO {:X}", data.to_u32());

                s.map.vi.regs[H_VIDEO_REG] = data;
            }

            V_VIDEO_REG => {
                // TODO
                log::warn!("Write VI_V_VIDEO {:X}", data.to_u32());

                s.map.vi.regs[V_VIDEO_REG] = data;
            }

            V_BURST_REG => {
                // TODO
                log::warn!("Write VI_V_BURST {:X}", data.to_u32());

                s.map.vi.regs[V_BURST_REG] = data;
            }

            X_SCALE_REG => {
                // TODO
                log::warn!("Write VI_X_SCALE {:X}", data.to_u32());

                s.map.vi.regs[X_SCALE_REG] = data;
            }

            Y_SCALE_REG => {
                // TODO
                //log::warn!("Write VI_Y_SCALE {:X}", data.to_u32());

                s.map.vi.regs[Y_SCALE_REG] = data;
            }

            _ => unimplemented!(
                "Write VI register {:X} @ {:08X}",
                data.to_u32(),
                addr.relative()
            ),
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

        s.map.vi.regs[CURRENT_SCANLINE_REG] += 1;

        if s.map.vi.regs[CURRENT_SCANLINE_REG] >= s.map.vi.regs[V_SYNC_REG] {
            s.map.vi.regs[CURRENT_SCANLINE_REG] = 0;
        }

        if s.map.vi.regs[CURRENT_SCANLINE_REG] == s.map.vi.regs[INTERRUPT_SCANLINE_REG] {
            s.map.mi.set_pending_interrupt(Interrupt::Vi);
        }

        s.events.push(Event {
            id: EventType::ViScanlineComplete,
            cycle: s.cycles + 1000, // TODO!!!
        });
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

    pub fn framebuffer_address(&self) -> u32 {
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

        for y in 0..height {
            for x in 0..width {
                let pixel: u32 = s.read(base_addr + ((y * width + x) * 4) as u32);

                data.push((pixel >> 24) as u8);
                data.push((pixel >> 16) as u8);
                data.push((pixel >> 8) as u8);
                data.push(0xFF); // TODO real val
            }
        }

        (data, width, height)
    }
}
