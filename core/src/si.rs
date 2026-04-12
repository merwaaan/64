//! Serial interface
//!
//! Handles DMA transfers between RAM and PIF RAM/ROM.
//! Typically used to communicate with the controllers.

use strum::{Display, EnumIter};

use crate::{
    events::{EventType, Events},
    location::Location,
    mi::Interrupt,
    pif::PifRamLocation,
    ram::RamLocation,
    system::System,
    value::Value,
};

pub type SiLocation = Location<0x0480_0000, 0x0490_0000>;

const MASK: u32 = 0x1F; // TODO?

const DRAM_ADDR_REG: usize = 0;
const PIF_ADDR_READ64_REG: usize = 1;
const PIF_ADDR_WRITE4_REG: usize = 2;
const PIF_ADDR_WRITE64_REG: usize = 4;
const PIF_ADDR_READ4_REG: usize = 5;
const STATUS_REG: usize = 6;

#[derive(Debug, Display, Clone, Copy, EnumIter)]
#[repr(u32)]
pub enum Register {
    DramAddr = DRAM_ADDR_REG as u32,
    PifAddrRead64 = PIF_ADDR_READ64_REG as u32,
    PifAddrWrite4 = PIF_ADDR_WRITE4_REG as u32,
    PifAddrWrite64 = PIF_ADDR_WRITE64_REG as u32,
    PifAddrRead4 = PIF_ADDR_READ4_REG as u32,
}

const STATUS_DMA_BUSY_MASK: u32 = 1;
const STATUS_IO_BUSY_MASK: u32 = 1 << 1;
const STATUS_READ_PENDING_MASK: u32 = 1 << 2;
const STATUS_DMA_ERROR_MASK: u32 = 1 << 3;
const STATUS_PCH_STATE_MASK: u32 = 0b1111 << 4;
const STATUS_DMA_STATE_MASK: u32 = 0b1111 << 8;
const STATUS_INTERRUPT_MASK: u32 = 1 << 12;

#[derive(Debug)]
enum DmaDirection {
    PifToRam,
    RamToPif,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Si {
    pub regs: [u32; 13],
}

impl Si {
    pub fn read<T: Value>(s: &System, addr: SiLocation) -> T {
        // TODO depends???

        // TODO temp
        if addr.relative() > 0x1B {
            panic!("Read invalid SI register @ {:08X}", addr.relative());
        }

        T::read_reg(&s.si.regs, addr.relative() & MASK)
    }

    pub fn write<T: Value>(s: &mut System, addr: SiLocation, data: T) {
        let reg = ((addr.relative() & MASK) >> 2) as usize;

        // TODO possible to write mult regs???
        debug_assert!(T::BYTES <= 4, "Writing to multiple SI registers");

        match reg {
            DRAM_ADDR_REG => {
                data.write_reg(&mut s.si.regs, addr.relative() & MASK);

                s.si.regs[DRAM_ADDR_REG] &= 0x00FF_FFFF;
            }
            PIF_ADDR_READ64_REG => {
                data.write_reg(&mut s.si.regs, addr.relative() & MASK);

                s.si.regs[PIF_ADDR_READ64_REG] &= 0x00FF_FFFC;

                Self::start_dma(s, DmaDirection::PifToRam);
            }
            PIF_ADDR_WRITE64_REG => {
                data.write_reg(&mut s.si.regs, addr.relative() & MASK);

                s.si.regs[PIF_ADDR_WRITE64_REG] &= 0x00FF_FFFC;

                Self::start_dma(s, DmaDirection::RamToPif);
            }
            STATUS_REG => {
                // Read-only but writing any value clears the interrupt

                s.si.regs[STATUS_REG] &= !STATUS_INTERRUPT_MASK;

                s.mi.clear_pending_interrupt(Interrupt::Si, &mut s.cop0);
            }
            _ => unimplemented!("Write SI register @ {:08X}", addr.relative()),
        }
    }

    fn start_dma(s: &mut System, dir: DmaDirection) {
        s.si.regs[STATUS_REG] |= STATUS_DMA_BUSY_MASK;
        // TODO IO busy?
        s.si.regs[STATUS_REG] &= !STATUS_DMA_ERROR_MASK; // TODO not needed if we never set it
        s.si.regs[STATUS_REG] |= STATUS_INTERRUPT_MASK;

        match dir {
            DmaDirection::PifToRam => {
                // log::info!(
                //     "SI DMA transfer: PIF {:08X} to RAM {:08X}",
                //     s.si.regs[PIF_ADDR_READ64_REG],
                //     s.si.regs[DRAM_ADDR_REG],
                // );

                s.pif.read_block(
                    &s.controllers,
                    PifRamLocation::from_relative(0),
                    0x40,
                    |pif_data| {
                        s.ram.write_block(
                            RamLocation::from_absolute(s.si.regs[DRAM_ADDR_REG]),
                            pif_data,
                        );
                    },
                );
            }
            DmaDirection::RamToPif => {
                // log::info!(
                //     "SI DMA transfer: RAM {:08X} to PIF {:08X}",
                //     s.si.regs[DRAM_ADDR_REG],
                //     s.si.regs[PIF_ADDR_READ64_REG]
                // );

                s.ram.read_block(
                    RamLocation::from_absolute(s.si.regs[DRAM_ADDR_REG]),
                    0x40,
                    |ram_data| {
                        s.pif
                            .write_block(PifRamLocation::from_relative(0), ram_data);
                    },
                );
            }
        }

        // TODO IO?

        Events::push(s, EventType::SiDmaTransferComplete, 10_000); // TODO which value?
    }

    pub fn dma_completed(s: &mut System) {
        // Update the status register

        s.si.regs[STATUS_REG] &= !STATUS_DMA_BUSY_MASK;
        // TODO IO busy?

        // Raise an SI interrupt

        s.mi.set_pending_interrupt(Interrupt::Si, &mut s.cop0);
    }
}
