use strum::{Display, EnumIter};

use crate::{data::Value, map::Location, mi::Interrupt, system::System};

const START: u32 = 0x0480_0000;
const END: u32 = 0x0490_0000;

pub type SiLocation = Location<START, END>;

const MASK: u32 = 0x1F; // TODO?

#[derive(Debug, Display, Clone, Copy, EnumIter)]
#[repr(u32)]
pub enum Register {
    DramAddr,
    PifAddrRead64,
    PifAddrRead4,
    PifAddrWrite64,
    PifAddrWrite4,
}

// TODO macro?

const DRAM_ADDR_REG: usize = 0;
const DRAM_ADDR_LO: u32 = (DRAM_ADDR_REG as u32) << 2;

const PIF_ADDR_READ64_REG: usize = 1;
const PIF_ADDR_READ64_LO: u32 = (PIF_ADDR_READ64_REG as u32) << 2;

const PIF_ADDR_READ4_REG: usize = 2;
const PIF_ADDR_READ4_LO: u32 = (PIF_ADDR_READ4_REG as u32) << 2;

const PIF_ADDR_WRITE64_REG: usize = 3;
const PIF_ADDR_WRITE64_LO: u32 = (PIF_ADDR_WRITE64_REG as u32) << 2;

const PIF_ADDR_WRITE4_REG: usize = 5;
const PIF_ADDR_WRITE4_LO: u32 = (PIF_ADDR_WRITE4_REG as u32) << 2;

const STATUS_REG: usize = 6;
const STATUS_LO: u32 = (STATUS_REG as u32) << 2;

//const STATUS_DMA_BUSY_MASK: u32 = 1;
//const STATUS_IO_BUSY_MASK: u32 = 1 << 1;
//const STATUS_READ_PENDING_MASK: u32 = 1 << 2;
//const STATUS_DMA_ERROR_MASK: u32 = 1 << 3;
// TODO others

#[derive(Default, Clone, Copy)]
pub struct Si {
    pub regs: [u32; 13],
}

impl Si {
    pub fn read<T: Value>(&self, addr: SiLocation) -> T {
        // TODO depends???

        // TODO temp
        if addr.relative() > 0x1B {
            panic!("Read invalid SI register @ {:08X}", addr.relative());
        }

        log::info!("Read SI register @ {:08X}", addr.relative());

        T::read_reg(&self.regs, addr.relative() & MASK)
    }

    pub fn write<T: Value>(s: &mut System, addr: SiLocation, data: T) {
        let reg = ((addr.relative() & MASK) >> 2) as usize;

        log::info!("Write SI register @ {:08X} {:X}", addr.relative(), data);

        match reg {
            DRAM_ADDR_REG => {
                //panic!("Write SI_DRAM_ADDR {:X}", data);

                data.write_reg(&mut s.map.si.regs, addr.relative() & MASK);

                s.map.si.regs[DRAM_ADDR_REG] &= 0x00FF_FFFF;
            }
            STATUS_REG => {
                // Writing any value acknowledges the interrupt

                s.map.mi.clear_pending_interrupt(Interrupt::Si);
            }
            _ => unimplemented!("Write SI register @ {:08X}", addr.relative()),
        }
    }

    pub fn reg_info(addr: SiLocation) -> Option<&'static str> {
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
