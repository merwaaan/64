use crate::{
    data::Data,
    events::{Event, EventType},
    mi::Interrupt,
    system::System,
};

pub const START: u32 = 0x0440_0000;
pub const SIZE: u32 = 0x10_0000;
pub const END: u32 = START + SIZE;

pub const MASK: u32 = 0x3F;

const STATUS_REG: usize = 0;
const STATUS_LO: u32 = (STATUS_REG as u32) << 2;
pub const STATUS: u32 = START | STATUS_LO;

// TODO flags

const FRAMEBUFFER_ADDR_REG: usize = 1; // "ORIGIN" in some docs
const FRAMEBUFFER_ADDR_LO: u32 = (FRAMEBUFFER_ADDR_REG as u32) << 2;
pub const FRAMEBUFFER_ADDR: u32 = START | FRAMEBUFFER_ADDR_LO;

const WIDTH_REG: usize = 2;
const WIDTH_LO: u32 = (WIDTH_REG as u32) << 2;
pub const WIDTH: u32 = START | WIDTH_LO;

const INTERRUPT_SCANLINE_REG: usize = 3;
const INTERRUPT_SCANLINE_LO: u32 = (INTERRUPT_SCANLINE_REG as u32) << 2;
pub const INTERRUPT_SCANLINE: u32 = START | INTERRUPT_SCANLINE_LO;

const CURRENT_SCANLINE_REG: usize = 4;
const CURRENT_SCANLINE_LO: u32 = (CURRENT_SCANLINE_REG as u32) << 2;
pub const CURRENT_SCANLINE: u32 = START | CURRENT_SCANLINE_LO;

const BURST_REG: usize = 5;
const BURST_LO: u32 = (BURST_REG as u32) << 2;
pub const BURST: u32 = START | BURST_LO;

const V_SYNC_REG: usize = 6;
const V_SYNC_LO: u32 = (V_SYNC_REG as u32) << 2;
pub const V_SYNC: u32 = START | V_SYNC_LO;

const H_SYNC_REG: usize = 7;
const H_SYNC_LO: u32 = (H_SYNC_REG as u32) << 2;
pub const H_SYNC: u32 = START | H_SYNC_LO;

const H_SYNC_LEAP_REG: usize = 8;
const H_SYNC_LEAP_LO: u32 = (H_SYNC_LEAP_REG as u32) << 2;
pub const H_SYNC_LEAP: u32 = START | H_SYNC_LEAP_LO;

const H_VIDEO_REG: usize = 9;
const H_VIDEO_LO: u32 = (H_VIDEO_REG as u32) << 2;
pub const H_VIDEO: u32 = START | H_VIDEO_LO;

const V_VIDEO_REG: usize = 10;
const V_VIDEO_LO: u32 = (V_VIDEO_REG as u32) << 2;
pub const V_VIDEO: u32 = START | V_VIDEO_LO;

const V_BURST_REG: usize = 11;
const V_BURST_LO: u32 = (V_BURST_REG as u32) << 2;
pub const V_BURST: u32 = START | V_BURST_LO;

const X_SCALE_REG: usize = 12;
const X_SCALE_LO: u32 = (X_SCALE_REG as u32) << 2;
pub const X_SCALE: u32 = START | X_SCALE_LO;

const Y_SCALE_REG: usize = 13;
const Y_SCALE_LO: u32 = (Y_SCALE_REG as u32) << 2;
pub const Y_SCALE: u32 = START | Y_SCALE_LO;

// NTSC 59.94 Hz, 262.5 scanlines
// PAL 50.00 Hz, 312.5 scanlines

pub struct Vi {
    regs: [u32; 13],
}

impl Default for Vi {
    fn default() -> Self {
        Self {
            regs: [
                0, 0, 0, 0, 0, 0, 0x271, // TODO PAL = 20D?
                0, 0, 0, 0, 0, 0,
            ],
        }
    }
}

impl Vi {
    pub fn read<T: Data>(&self, addr: u32) -> T {
        assert_range(addr);

        let reg = ((addr & MASK) >> 2) as usize;

        match reg {
            FRAMEBUFFER_ADDR_REG => T::from_u32(self.regs[FRAMEBUFFER_ADDR_REG]),

            // TODO half scanlines???
            CURRENT_SCANLINE_REG => T::from_u32(self.regs[CURRENT_SCANLINE_REG]),

            _ => unimplemented!("Read VI register @ {:08X}", addr),
        }
    }

    pub fn write<T: Data>(s: &mut System, addr: u32, data: T) {
        assert_range(addr);

        let reg = ((addr & MASK) >> 2) as usize;

        let data = data.to_u32(); // TODO temp hack, should be able to write any size

        match reg {
            STATUS_REG => {
                // TODO
                log::warn!("Write VI_STATUS {:X}", data.to_u32());
            }

            FRAMEBUFFER_ADDR_REG => {
                // TODO
                log::warn!("Write VI_FRAMEBUFFER_ADDR {:X}", data.to_u32());
            }

            WIDTH_REG => {
                // TODO
                log::warn!("Write VI_WIDTH {:X}", data.to_u32());
            }

            INTERRUPT_SCANLINE_REG => {
                // TODO
                log::warn!("Write VI_INTERRUPT_SCANLINE {:X}", data.to_u32());
            }

            CURRENT_SCANLINE_REG => {
                // Writing anything to this register clears the Interrupt and resets the current scanline

                s.map.mi.clear_pending_interrupt(Interrupt::Vi);

                s.map.vi.regs[CURRENT_SCANLINE_REG] = 0; // TODO really?
            }

            BURST_REG => {
                // TODO
                log::warn!("Write VI_BURST {:X}", data.to_u32());
            }

            V_SYNC_REG => {
                // TODO
                log::warn!("Write VI_V_SYNC {:X}", data.to_u32());
            }

            H_SYNC_REG => {
                // TODO
                log::warn!("Write VI_H_SYNC {:X}", data.to_u32());
            }

            H_SYNC_LEAP_REG => {
                // TODO
                log::warn!("Write VI_H_SYNC_LEAP {:X}", data.to_u32());
            }

            H_VIDEO_REG => {
                // TODO
                log::warn!("Write VI_H_VIDEO {:X}", data.to_u32());
            }

            V_VIDEO_REG => {
                // TODO
                log::warn!("Write VI_V_VIDEO {:X}", data.to_u32());
            }

            V_BURST_REG => {
                // TODO
                log::warn!("Write VI_V_BURST {:X}", data.to_u32());
            }

            X_SCALE_REG => {
                // TODO
                log::warn!("Write VI_X_SCALE {:X}", data.to_u32());
            }

            Y_SCALE_REG => {
                // TODO
                log::warn!("Write VI_Y_SCALE {:X}", data.to_u32());
            }

            _ => unimplemented!("Write VI register {:X} @ {:08X}", data.to_u32(), addr),
        }
    }

    // fn start_dma(s: &mut System) {
    //     // Instant DMA transfer!
    //     // TODO make it progressive?

    //     let length = s.map.pi.regs[WR_LEN_REG] + 1;

    //     log::warn!(
    //         "PI DMA transfer: {:#X} from {:#X} to {:#X} @ {}",
    //         length,
    //         s.map.pi.regs[CART_ADDR_REG],
    //         s.map.pi.regs[DRAM_ADDR_REG],
    //         s.cpu.step,
    //     );

    //     for offset in 0..length {
    //         let data: u32 = s.read(s.map.pi.regs[CART_ADDR_REG] + offset);

    //         s.write(s.map.pi.regs[DRAM_ADDR_REG] + offset, data);
    //     }

    //     // Update the status register

    //     s.map.pi.regs[STATUS_REG] |= STATUS_DMA_BUSY_MASK;
    //     // TODO IO busy?
    //     // TODO DMA error? if already busy?

    //     // TODO schedule status update

    //     s.events.push(Event {
    //         id: EventType::PiDmaTransferComplete,
    //         cycle: s.cycles + (length / 8 + 100/* TODO temp hack to match pj */) as usize,
    //     });
    // }

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
            s.map.mi.set_pending_interrupt(Interrupt::Pi);
        }

        s.events.push(Event {
            id: EventType::ViScanlineComplete,
            cycle: s.cycles + 1000, // TODO!!!
        });
    }

    pub fn address_info(addr: u32) -> Option<&'static str> {
        assert_range(addr);

        // TODO check masks!
        // TODO normalize strings

        let s = match addr & MASK {
            STATUS_LO => "VI_STATUS",
            FRAMEBUFFER_ADDR_LO => "VI_FRAMEBUFFER_ADDR",
            WIDTH_LO => "VI_WIDTH",
            INTERRUPT_SCANLINE_LO => "VI_INTERRUPT_SCANLINE",
            // TODO others
            _ => "???", // TODO
        };

        // TODO cleaner way to do that?
        if s.is_empty() { None } else { Some(s) }
    }
}

fn assert_range(addr: u32) {
    debug_assert!((START..END).contains(&addr));
}
