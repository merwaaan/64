use crate::{data::Data, map::Location, mi::Interrupt, system::System};

pub const START: u32 = 0x0480_0000;
pub const SIZE: u32 = 0x10_0000;
pub const END: u32 = START + SIZE;

pub type SiLocation = Location<START, END>;

pub const MASK: u32 = 0x1F; // TODO?

// TODO macro?

const DRAM_ADDR_REG: usize = 0;
const DRAM_ADDR_LO: u32 = (DRAM_ADDR_REG as u32) << 2;
pub const DRAM_ADDR: u32 = START | DRAM_ADDR_LO;

const PIF_ADDR_READ64_REG: usize = 1;
const PIF_ADDR_READ64_LO: u32 = (PIF_ADDR_READ64_REG as u32) << 2;
pub const PIF_ADDR_READ64: u32 = START | PIF_ADDR_READ64_LO;

const PIF_ADDR_READ4_REG: usize = 2;
const PIF_ADDR_READ4_LO: u32 = (PIF_ADDR_READ4_REG as u32) << 2;
pub const PIF_ADDR_READ4: u32 = START | PIF_ADDR_READ4_LO;

const PIF_ADDR_WRITE64_REG: usize = 3;
const PIF_ADDR_WRITE64_LO: u32 = (PIF_ADDR_WRITE64_REG as u32) << 2;
pub const PIF_ADDR_WRITE64: u32 = START | PIF_ADDR_WRITE64_LO;

const PIF_ADDR_WRITE4_REG: usize = 5;
const PIF_ADDR_WRITE4_LO: u32 = (PIF_ADDR_WRITE4_REG as u32) << 2;
pub const PIF_ADDR_WRITE4: u32 = START | PIF_ADDR_WRITE4_LO;

const STATUS_REG: usize = 6;
const STATUS_LO: u32 = (STATUS_REG as u32) << 2;
pub const STATUS: u32 = START | STATUS_LO;

const STATUS_DMA_BUSY_MASK: u32 = 1;
const STATUS_IO_BUSY_MASK: u32 = 1 << 1;
const STATUS_READ_PENDING_MASK: u32 = 1 << 2;
const STATUS_DMA_ERROR_MASK: u32 = 1 << 3;
// TODO others

#[derive(Default)]
pub struct Si {
    regs: [u32; 13],
}

impl Si {
    pub fn read<T: Data>(&self, addr: SiLocation) -> T {
        let reg = ((addr.relative() & MASK) >> 2) as usize;

        // TODO depends???

        T::from_u32(self.regs[reg as usize]) // TOD0 weirddd
    }

    pub fn write<T: Data>(s: &mut System, addr: SiLocation, data: T) {
        let reg = ((addr.relative() & MASK) >> 2) as usize;

        let data = data.to_u32(); // TODO temp hack, should be able to write any size

        match reg {
            DRAM_ADDR_REG => {
                //panic!("Write SI_DRAM_ADDR {:X}", data);
                s.map.si.regs[DRAM_ADDR_REG] = data & 0x00FF_FFFF;
            }
            STATUS_REG => {
                // Writing any value acknowledges the interrupt

                s.map.mi.clear_pending_interrupt(Interrupt::Si);
            }
            _ => unimplemented!("Write SI register @ {:08X}", addr.relative()),
        }
    }

    pub fn address_info(addr: SiLocation) -> Option<&'static str> {
        // TODO check masks!
        // TODO normalize strings

        let s = match addr.relative() & MASK {
            DRAM_ADDR_LO => "SI_DRAM_ADDR",
            PIF_ADDR_READ64_LO => "SI_PIF_ADDR_READ64",
            PIF_ADDR_READ4_LO => "SI_PIF_ADDR_READ4",
            PIF_ADDR_WRITE64_LO => "SI_PIF_ADDR_WRITE64",
            PIF_ADDR_WRITE4_LO => "SI_PIF_ADDR_WRITE4",
            STATUS_LO => "SI_STATUS",
            _ => "???", // TODO
        };

        // TODO cleaner way to do that?
        if s.is_empty() { None } else { Some(s) }
    }
}
