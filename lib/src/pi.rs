use crate::{
    data::Data,
    events::{Event, EventType},
    system::System,
};

pub const PI_START: u32 = 0x0460_0000;
pub const PI_SIZE: u32 = 0x10_0000;
pub const PI_END: u32 = PI_START + PI_SIZE;

pub const PI_MASK: u32 = 0x1F;

// TODO macro?

const PI_DRAM_ADDR_REG: usize = 0;
const PI_DRAM_ADDR_LO: u32 = (PI_DRAM_ADDR_REG as u32) << 2;
pub const PI_DRAM_ADDR: u32 = PI_START | PI_DRAM_ADDR_LO;

const PI_CART_ADDR_REG: usize = 1;
const PI_CART_ADDR_LO: u32 = (PI_CART_ADDR_REG as u32) << 2;
pub const PI_CART_ADDR: u32 = PI_START | PI_CART_ADDR_LO;

const PI_RD_LEN_REG: usize = 2;
const PI_RD_LEN_LO: u32 = (PI_RD_LEN_REG as u32) << 2;
pub const PI_RD_LEN: u32 = PI_START | PI_RD_LEN_LO;

const PI_WR_LEN_REG: usize = 3;
pub const PI_WR_LEN_LO: u32 = (PI_WR_LEN_REG as u32) << 2;
const PI_WR_LEN: u32 = PI_START | PI_WR_LEN_LO;

const PI_STATUS_REG: usize = 4;
const PI_STATUS_LO: u32 = (PI_STATUS_REG as u32) << 2;
pub const PI_STATUS: u32 = PI_START | PI_STATUS_LO;

const PI_STATUS_DMA_BUSY_MASK: u32 = 1;
const PI_STATUS_IO_BUSY_MASK: u32 = 2;
const PI_STATUS_DMA_ERROR_MASK: u32 = 4;
const PI_STATUS_DMA_COMPLETED_MASK: u32 = 8;

#[derive(Default)]
pub struct Pi {
    regs: [u32; 13],
}

impl Pi {
    pub fn read<T: Data>(&self, addr: u32) -> T {
        assert_range(addr);

        let reg = (addr & PI_MASK) >> 2;

        // TODO depends???

        T::from_u32(self.regs[reg as usize]) // TOD0 weirddd
    }

    pub fn write<T: Data>(s: &mut System, addr: u32, data: T) {
        assert_range(addr);

        let reg = ((addr & PI_MASK) >> 2) as usize;

        let data = data.to_u32(); // TODO temp hack, should be able to write any size

        match reg {
            PI_DRAM_ADDR_REG => {
                s.map.pi.regs[PI_DRAM_ADDR_REG] = data & 0x00FF_FFFE;
            }
            PI_CART_ADDR_REG => {
                s.map.pi.regs[PI_CART_ADDR_REG] = data & 0xFFFF_FFFE; // TODO auto updated after DMA transfer
            }
            PI_RD_LEN_REG => {
                s.map.pi.regs[PI_RD_LEN_REG] = data & 0x00FF_FFFF;

                unimplemented!("Write to PI_RD_LEN");
            }
            PI_WR_LEN_REG => {
                s.map.pi.regs[PI_WR_LEN_REG] = data & 0x00FF_FFFF;

                Self::start_dma(s);
            }
            PI_STATUS_REG => {
                // Bit 1: clear the interrupt bit
                if (data & 2) == 2 {
                    s.map.pi.regs[PI_STATUS_REG] &= !PI_STATUS_DMA_COMPLETED_MASK;
                }

                // Bit 0: clear the error bit
                if (data & 1) == 1 {
                    s.map.pi.regs[PI_STATUS_REG] &= !PI_STATUS_DMA_ERROR_MASK;
                }
            }
            _ => unimplemented!("Write PI register @ {:08X}", addr),
        }
    }

    fn start_dma(s: &mut System) {
        // Instant DMA transfer!
        // TODO make it progressive?

        let length = s.map.pi.regs[PI_WR_LEN_REG] + 1;

        log::warn!(
            "PI DMA transfer: {:#X} from {:#X} to {:#X} @ {}",
            length,
            s.map.pi.regs[PI_CART_ADDR_REG],
            s.map.pi.regs[PI_DRAM_ADDR_REG],
            s.cpu.step,
        );

        for offset in 0..length {
            let data: u32 = s.read(s.map.pi.regs[PI_CART_ADDR_REG] + offset);

            s.write(s.map.pi.regs[PI_DRAM_ADDR_REG] + offset, data);
        }

        // Update the status register

        s.map.pi.regs[PI_STATUS_REG] |= PI_STATUS_DMA_BUSY_MASK;
        // TODO IO busy?
        // TODO DMA error? if already busy?

        // TODO schedule status update

        s.events.push(Event {
            id: EventType::PiDmaTransferComplete,
            cycle: s.cycles + (length / 8 + 100/* TODO temp hack to match pj */) as usize,
        });
    }

    pub fn dma_completed(s: &mut System) {
        s.map.pi.regs[PI_STATUS_REG] |= PI_STATUS_DMA_COMPLETED_MASK;
        s.map.pi.regs[PI_STATUS_REG] &= !PI_STATUS_DMA_BUSY_MASK;
        // TODO IO busy?
    }

    pub fn address_info(addr: u32) -> Option<&'static str> {
        assert_range(addr);

        // TODO check masks!
        // TODO normalize strings

        let s = match addr & PI_MASK {
            PI_DRAM_ADDR_LO => "PI_DRAM_ADDR",
            PI_CART_ADDR_LO => "PI_CART_ADDR",
            PI_RD_LEN_LO => "PI_RD_LEN",
            PI_WR_LEN_LO => "PI_WR_LEN",
            PI_STATUS_LO => "PI_STATUS",
            // 0x14 => "PI_BSD_DOM1_LAT",
            // 0x18 => "PI_BSD_DOM1_PWD",
            // 0x20 => "PI_BSD_DOM1_RLS",
            // 0x24 => "PI_BSD_DOM2_LAT",
            // 0x28 => "PI_BSD_DOM2_PWD",
            // 0x1C => "PI_BSD_DOM1_PGS",
            // 0x2C => "PI_BSD_DOM2_PGS",
            // 0x30 => "PI_BSD_DOM2_RLS",
            _ => "PI_???",
        };

        // TODO cleaner way to do that?
        if s.is_empty() { None } else { Some(s) }
    }
}

fn assert_range(addr: u32) {
    debug_assert!((PI_START..PI_END).contains(&addr));
}
