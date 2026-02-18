use crate::{
    data::Data,
    events::{Event, EventType},
    map::Location,
    mi::Interrupt,
    system::System,
};

pub const START: u32 = 0x0460_0000;
pub const SIZE: u32 = 0x10_0000;
pub const END: u32 = START + SIZE;

pub type PiLocation = Location<START, END>;

pub const MASK: u32 = 0x1F;

// TODO macro?

const DRAM_ADDR_REG: usize = 0;
const DRAM_ADDR_LO: u32 = (DRAM_ADDR_REG as u32) << 2;
pub const DRAM_ADDR: u32 = START | DRAM_ADDR_LO;

const CART_ADDR_REG: usize = 1;
const CART_ADDR_LO: u32 = (CART_ADDR_REG as u32) << 2;
pub const CART_ADDR: u32 = START | CART_ADDR_LO;

const READ_LEN_REG: usize = 2;
const READ_LEN_LO: u32 = (READ_LEN_REG as u32) << 2;
pub const READ_LEN: u32 = START | READ_LEN_LO;

const WRITE_LEN_REG: usize = 3;
pub const WRITE_LEN_LO: u32 = (WRITE_LEN_REG as u32) << 2;
const WRITE_LEN: u32 = START | WRITE_LEN_LO;

const STATUS_REG: usize = 4;
const STATUS_LO: u32 = (STATUS_REG as u32) << 2;
pub const STATUS: u32 = START | STATUS_LO;

const STATUS_DMA_BUSY_MASK: u32 = 1;
const STATUS_IO_BUSY_MASK: u32 = 1 << 1;
const STATUS_DMA_ERROR_MASK: u32 = 1 << 2;
const STATUS_DMA_COMPLETED_MASK: u32 = 1 << 3;

#[derive(Default)]
pub struct Pi {
    regs: [u32; 13],
}

impl Pi {
    pub fn read<T: Data>(&self, addr: PiLocation) -> T {
        let reg = ((addr.relative() & MASK) >> 2) as usize;

        // TODO depends???

        T::from_u32(self.regs[reg]) // TOD0 weirddd
    }

    pub fn write<T: Data>(s: &mut System, addr: PiLocation, data: T) {
        let reg = ((addr.relative() & MASK) >> 2) as usize;

        let data = data.to_u32(); // TODO temp hack, should be able to write any size

        match reg {
            DRAM_ADDR_REG => {
                s.map.pi.regs[DRAM_ADDR_REG] = data & 0x00FF_FFFE;
            }
            CART_ADDR_REG => {
                s.map.pi.regs[CART_ADDR_REG] = data & 0xFFFF_FFFE; // TODO auto updated after DMA transfer
            }
            READ_LEN_REG => {
                s.map.pi.regs[READ_LEN_REG] = data & 0x00FF_FFFF;

                unimplemented!("Write to READ_LEN");
            }
            WRITE_LEN_REG => {
                s.map.pi.regs[WRITE_LEN_REG] = data & 0x00FF_FFFF;

                Self::start_dma(s);
            }
            STATUS_REG => {
                // Bit 1: clear the interrupt
                if (data & 2) == 2 {
                    s.map.pi.regs[STATUS_REG] &= !STATUS_DMA_COMPLETED_MASK;
                    s.map.mi.clear_pending_interrupt(Interrupt::Pi);
                }

                // Bit 0: clear the error
                if (data & 1) == 1 {
                    s.map.pi.regs[STATUS_REG] &= !STATUS_DMA_ERROR_MASK;
                }
            }
            _ => unimplemented!("Write PI register @ {:08X}", addr.relative()),
        }
    }

    fn start_dma(s: &mut System) {
        // Instant DMA transfer

        let length = s.map.pi.regs[WRITE_LEN_REG] + 1;

        log::info!(
            "PI DMA transfer: {} bytes from {:08X} to {:08X}",
            length,
            s.map.pi.regs[CART_ADDR_REG],
            s.map.pi.regs[DRAM_ADDR_REG]
        );

        let dest_base = s.map.pi.regs[DRAM_ADDR_REG];

        for offset in 0..length {
            let data = s.read::<u8>(s.map.pi.regs[CART_ADDR_REG] + offset);

            // log::info!(
            //     "PI DMA transfer: offset {:08X} data {:02X}",
            //     dest_base + offset,
            //     data
            // );

            s.write::<u8>(dest_base + offset, data);
        }

        // Update the status register

        s.map.pi.regs[STATUS_REG] |= STATUS_DMA_BUSY_MASK;
        // TODO IO busy?
        // TODO DMA error? if already busy?

        // TODO schedule status update

        s.events.push(Event {
            id: EventType::PiDmaTransferComplete,
            cycle: s.cycles
                + (
                    length / 8
                    /*+ 100*//* TODO temp hack to match pj */
                ) as usize,
        });
    }

    pub fn dma_completed(s: &mut System) {
        // Update the status register

        s.map.pi.regs[STATUS_REG] |= STATUS_DMA_COMPLETED_MASK;
        s.map.pi.regs[STATUS_REG] &= !STATUS_DMA_BUSY_MASK;
        // TODO IO busy?

        // Raise the interrupt

        s.map.mi.set_pending_interrupt(Interrupt::Pi);
    }

    pub fn address_info(addr: PiLocation) -> Option<&'static str> {
        // TODO check masks!
        // TODO normalize strings

        let s = match addr.relative() & MASK {
            DRAM_ADDR_LO => "PI_DRAM_ADDR",
            CART_ADDR_LO => "PI_CART_ADDR",
            READ_LEN_LO => "PI_READ_LEN",
            WRITE_LEN_LO => "PI_WRITE_LEN",
            STATUS_LO => "PI_STATUS",
            // 0x14 => "BSD_DOM1_LAT",
            // 0x18 => "BSD_DOM1_PWD",
            // 0x20 => "BSD_DOM1_RLS",
            // 0x24 => "BSD_DOM2_LAT",
            // 0x28 => "BSD_DOM2_PWD",
            // 0x1C => "BSD_DOM1_PGS",
            // 0x2C => "BSD_DOM2_PGS",
            // 0x30 => "BSD_DOM2_RLS",
            _ => "???", // TODO
        };

        // TODO cleaner way to do that?
        if s.is_empty() { None } else { Some(s) }
    }
}
