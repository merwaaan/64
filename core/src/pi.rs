use strum::{Display, EnumIter};

use crate::{
    cart::CartLocation,
    events::{EventType, Events},
    location::Location,
    mi::Interrupt,
    ram::RamLocation,
    system::System,
    value::Value,
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
const STATUS_IO_BUSY_MASK: u32 = 1 << 1;
const STATUS_DMA_ERROR_MASK: u32 = 1 << 2;
const STATUS_DMA_COMPLETED_MASK: u32 = 1 << 3;

#[derive(Default, Clone, Copy, Debug)]
pub struct Pi {
    pub regs: [u32; 13], // TODO not pub
}

impl Pi {
    pub fn read<T: Value>(s: &System, addr: PiLocation) -> T {
        // TODO depends???

        // TODO temp
        if addr.relative() > 0x13 {
            log::warn!("PI: read {:08X}", addr.relative());
        }

        T::read_reg(&s.pi.regs, addr.relative() & MASK)
    }

    pub fn write<T: Value>(s: &mut System, addr: PiLocation, data: T) {
        let reg = ((addr.relative() & MASK) >> 2) as usize;

        //log::info!("PI: write {:X?} to {:08X}", data, addr.relative());

        // TODO possible to write mult regs???
        debug_assert!(T::BYTES <= 4, "Writing to multiple PI registers");

        match reg {
            DRAM_ADDR_REG => {
                data.write_reg(&mut s.pi.regs, addr.relative() & MASK);

                s.pi.regs[DRAM_ADDR_REG] &= 0x00FF_FFFE;
            }
            CART_ADDR_REG => {
                data.write_reg(&mut s.pi.regs, addr.relative() & MASK);

                s.pi.regs[CART_ADDR_REG] &= 0xFFFF_FFFE;
            }
            READ_LEN_REG => {
                data.write_reg(&mut s.pi.regs, addr.relative() & MASK);

                s.pi.regs[READ_LEN_REG] &= 0x00FF_FFFF;

                log::warn!("PI: UNIMPLEMENTED write to READ_LEN");
            }
            WRITE_LEN_REG => {
                data.write_reg(&mut s.pi.regs, addr.relative() & MASK);

                s.pi.regs[WRITE_LEN_REG] &= 0x00FF_FFFF;

                Self::start_dma(s);
            }
            STATUS_REG => {
                let mut trigger_bits = [0u32];
                data.write_reg(&mut trigger_bits, addr.relative() & 3);

                // Bit 1: clear the interrupt

                if (trigger_bits[0] & 2) != 0 {
                    s.pi.regs[STATUS_REG] &= !STATUS_DMA_COMPLETED_MASK;
                    s.mi.clear_pending_interrupt(Interrupt::Pi, &mut s.cop0);
                }

                // Bit 0: clear the error

                if (trigger_bits[0] & 1) != 0 {
                    s.pi.regs[STATUS_REG] &= !STATUS_DMA_ERROR_MASK;
                }
            }
            _ => {
                log::warn!("PI: write {:X?} to {:08X}", data, addr.relative());

                data.write_reg(&mut s.pi.regs, addr.relative() & MASK);
            }
        }
    }

    fn start_dma(s: &mut System) {
        // PI DMA transfers have quirky behaviors for small sizes (< 128 bytes)
        // - length, if odd, is NOT rounded up to the next even value
        // - TODO what else???
        //
        // https://n64brew.dev/wiki/Parallel_Interface#Unaligned_DMA_transfer

        let requested_length = s.pi.regs[WRITE_LEN_REG] + 1;
        let misalignment = s.pi.regs[DRAM_ADDR_REG] & 7;

        // TODO still unclear!

        // let cart_length = if requested_length >= 0x7E {
        //     // TODO mis in cond
        //     (requested_length + 1) & !1
        // } else {
        //     requested_length
        // };

        // let ram_length = if requested_length >= 0x7E {
        //     (requested_length + 1) & !1
        // } else {
        //     requested_length.saturating_sub(misalignment)
        // };

        let cart_length = if (requested_length as i32) >= (126 - (misalignment as i32)) {
            //TODO u32 bug???
            (requested_length + 1) & !1
        } else {
            requested_length
        };

        let actual_length = if (requested_length as i32) < 128 - (misalignment as i32) {
            //TODO u32 bug???
            cart_length.saturating_sub(misalignment)
        } else {
            cart_length
        };

        // log::info!(
        //     "PI DMA transfer: {:X} bytes from CART {:08X} to RAM {:08X}",
        //     actual_length,
        //     s.pi.regs[CART_ADDR_REG],
        //     s.pi.regs[DRAM_ADDR_REG]
        // );

        let mut ram_offset = s.pi.regs[DRAM_ADDR_REG];

        s.cart.read_block(
            CartLocation::from_absolute(s.pi.regs[CART_ADDR_REG]),
            actual_length as usize,
            |cart_data| {
                s.ram
                    .write_block(RamLocation::from_absolute(ram_offset), cart_data);

                ram_offset = ram_offset.wrapping_add(cart_data.len() as u32);
            },
        );

        // Increment the addresses for the next transfer
        //
        // Fix misalignment:
        // - RAM: aligned to 8 bytes
        // - CART: aligned to 2 bytes

        // log::info!("PI DMA RAM address: {:08X}", s.pi.regs[DRAM_ADDR_REG]);

        s.pi.regs[DRAM_ADDR_REG] = s.pi.regs[DRAM_ADDR_REG]
            .wrapping_add(((actual_length + misalignment + 7) & !7) - misalignment); // TODO should mask?

        // log::info!(
        //     "ssssssssssssss: {}, {}, {}, {}",
        //     requested_length,
        //     actual_length,
        //     misalignment,
        //     ((actual_length + misalignment + 7) & !7) - misalignment
        // );
        // log::info!("PI DMA RAM address 2: {:08X}", s.pi.regs[DRAM_ADDR_REG]);

        s.pi.regs[CART_ADDR_REG] =
            s.pi.regs[CART_ADDR_REG].wrapping_add((requested_length + 1) & !1); // TODO should mask?

        // Update the status

        s.pi.regs[STATUS_REG] |= STATUS_DMA_BUSY_MASK;
        s.pi.regs[STATUS_REG] |= STATUS_IO_BUSY_MASK; // TODO not sure?
        s.pi.regs[STATUS_REG] &= !STATUS_DMA_ERROR_MASK;
        s.pi.regs[STATUS_REG] &= !STATUS_DMA_COMPLETED_MASK;
        // TODO DMA error? if already busy?

        // Schedule completion

        Events::push(
            s,
            EventType::PiDmaTransferComplete,
            (requested_length as usize) * 10, // TODO depends on the regs? TODO aligned length?
        );
    }

    pub fn dma_completed(s: &mut System) {
        // Update the status register

        s.pi.regs[STATUS_REG] |= STATUS_DMA_COMPLETED_MASK;
        s.pi.regs[STATUS_REG] &= !STATUS_DMA_ERROR_MASK;
        s.pi.regs[STATUS_REG] &= !STATUS_IO_BUSY_MASK; // TODO not sure?
        s.pi.regs[STATUS_REG] &= !STATUS_DMA_BUSY_MASK;
        // TODO IO busy?

        // Raise the interrupt

        s.mi.set_pending_interrupt(Interrupt::Pi, &mut s.cop0);
    }
}
