use strum::{Display, EnumIter};

use crate::{
    data::Value,
    events::{EventType, Events},
    map::Location,
    mi::Interrupt,
    system::System,
};

/// Peripheral interface
///
/// Handles DMA transfers between RAM and Cartridge.

const START: u32 = 0x0460_0000;
const END: u32 = 0x0470_0000;

pub type PiLocation = Location<START, END>;

const MASK: u32 = 0x3F;

#[derive(Debug, Display, Clone, Copy, EnumIter)]
#[repr(u32)]
pub enum Register {
    DramAddr,
    CartAddr,
    ReadLen,
    WriteLen,
    Status,
    Dom1Lat,
    Dom1Pwd,
    Dom1Pgs,
    Dom1Rls,
    Dom2Lat,
    Dom2Pwd,
    Dom2Pgs,
    Dom2Rls,
}

// TODO rm?
const DRAM_ADDR_REG: usize = 0;
const DRAM_ADDR_LO: u32 = (DRAM_ADDR_REG as u32) << 2;

const CART_ADDR_REG: usize = 1;
const CART_ADDR_LO: u32 = (CART_ADDR_REG as u32) << 2;

const READ_LEN_REG: usize = 2;
const READ_LEN_LO: u32 = (READ_LEN_REG as u32) << 2;

const WRITE_LEN_REG: usize = 3;
const WRITE_LEN_LO: u32 = (WRITE_LEN_REG as u32) << 2;

const STATUS_REG: usize = 4;
const STATUS_LO: u32 = (STATUS_REG as u32) << 2;

const STATUS_DMA_BUSY_MASK: u32 = 1;
//const STATUS_IO_BUSY_MASK: u32 = 1 << 1;
const STATUS_DMA_ERROR_MASK: u32 = 1 << 2;
const STATUS_DMA_COMPLETED_MASK: u32 = 1 << 3;

#[derive(Default)]
pub struct Pi {
    regs: [u32; 13],
}

impl Pi {
    pub fn read<T: Value>(&self, addr: PiLocation) -> T {
        // TODO depends???

        // TODO temp
        if addr.relative() > 0x13 {
            log::warn!("Read to PI register {:08X}", addr.relative());
        }

        T::read_reg(&self.regs, addr.relative() & MASK)
    }

    pub fn write<T: Value>(s: &mut System, addr: PiLocation, data: T) {
        let reg = ((addr.relative() & MASK) >> 2) as usize;

        match reg {
            DRAM_ADDR_REG => {
                log::error!("Write DRAM_ADDR {:X} @ {:08X}", data, addr.relative());
                data.write_reg(&mut s.map.pi.regs, addr.relative() & MASK);

                s.map.pi.regs[DRAM_ADDR_REG] &= 0x00FF_FFFE;
            }
            CART_ADDR_REG => {
                data.write_reg(&mut s.map.pi.regs, addr.relative() & MASK);

                s.map.pi.regs[CART_ADDR_REG] &= 0xFFFF_FFFE; // TODO auto updated after DMA transfer
            }
            READ_LEN_REG => {
                data.write_reg(&mut s.map.pi.regs, addr.relative() & MASK);

                s.map.pi.regs[READ_LEN_REG] &= 0x00FF_FFFF;

                unimplemented!("Write to READ_LEN");
            }
            WRITE_LEN_REG => {
                data.write_reg(&mut s.map.pi.regs, addr.relative() & MASK);

                s.map.pi.regs[WRITE_LEN_REG] &= 0x00FF_FFFF;

                Self::start_dma(s);
            }
            STATUS_REG => {
                let mut trigger_bits = [0u32];
                data.write_reg(&mut trigger_bits, addr.relative() & 3);

                // Bit 1: clear the interrupt

                if (trigger_bits[0] & 2) != 0 {
                    s.map.pi.regs[STATUS_REG] &= !STATUS_DMA_COMPLETED_MASK;
                    s.map.mi.clear_pending_interrupt(Interrupt::Pi, &mut s.cop0);
                }

                // Bit 0: clear the error

                if (trigger_bits[0] & 1) != 0 {
                    s.map.pi.regs[STATUS_REG] &= !STATUS_DMA_ERROR_MASK;
                }
            }
            _ => {
                log::warn!("Write {:X?} to PI register {:08X}", data, addr.relative());

                data.write_reg(&mut s.map.pi.regs, addr.relative() & MASK);
            }
        }
    }

    fn start_dma(s: &mut System) {
        // Instant DMA transfer

        let length = s.map.pi.regs[WRITE_LEN_REG] + 1;

        log::info!(
            "PI DMA transfer: {} bytes from CART {:08X} to RAM {:08X}",
            length,
            s.map.pi.regs[CART_ADDR_REG],
            s.map.pi.regs[DRAM_ADDR_REG]
        );

        let dest_base = s.map.pi.regs[DRAM_ADDR_REG];

        for offset in 0..length {
            let data = s.read::<u8>(s.map.pi.regs[CART_ADDR_REG] + offset);

            s.write::<u8>(dest_base + offset, data);
        }

        s.map.pi.regs[DRAM_ADDR_REG] = s.map.pi.regs[DRAM_ADDR_REG].wrapping_add(length);

        s.map.pi.regs[CART_ADDR_REG] = s.map.pi.regs[CART_ADDR_REG].wrapping_add(length);

        // Update the status register

        s.map.pi.regs[STATUS_REG] |= STATUS_DMA_BUSY_MASK;
        // TODO IO busy?
        // TODO DMA error? if already busy?

        // TODO schedule status update

        Events::push(
            s,
            EventType::PiDmaTransferComplete,
            (
                length / 8
                /*+ 100*//* TODO temp hack to match pj */
            ) as usize,
        );
    }

    pub fn dma_completed(s: &mut System) {
        // Update the status register

        s.map.pi.regs[STATUS_REG] |= STATUS_DMA_COMPLETED_MASK;
        s.map.pi.regs[STATUS_REG] &= !STATUS_DMA_BUSY_MASK;
        // TODO IO busy?

        // Raise the interrupt

        s.map.mi.set_pending_interrupt(Interrupt::Pi, &mut s.cop0);
    }

    pub fn reg_info(addr: PiLocation) -> Option<&'static str> {
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
