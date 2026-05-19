//! Parallel interface.
//! Primarily deals with communication with external devices like the cartridge.
//!
//! https://n64brew.dev/wiki/Parallel_Interface

use arbitrary_int::prelude::*;
use bitbybit::bitfield;

use crate::mapped_registers;

pub const START: u32 = 0x0460_0000;
pub const END: u32 = 0x0470_0000;

pub const DMA_RAM_ALIGNMENT: usize = 8;
pub const DMA_PI_ALIGNMENT: usize = 2;

/// Physical address in the main RAM to transfer data from/to via DMA.
#[bitfield(u32, instrospect, forbid_overlaps, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct DmaRamAddress {
    // TODO bit 0?
    #[bits(0..=23, rw)]
    value: u24,
}

/// Physical address in the PI bus  to transfer data from/to via DMA.
#[bitfield(u32, instrospect, forbid_overlaps, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct DmaPiAddress {
    // TODO bit 0?
    #[bits(0..=31, rw)]
    value: u32,
}

/// Writing to this register starts a transfer from RAM to PI.
#[bitfield(u32, instrospect, forbid_overlaps, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct DmaReadLength {
    /// Length of the transfer, minus 1.
    #[bits(0..=23, rw)]
    value: u24,
}

/// Writing to this register starts a transfer from PI to RAM.
#[bitfield(u32, instrospect, forbid_overlaps, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct DmaWriteLength {
    /// Length of the transfer, minus 1.
    #[bits(0..=23, rw)]
    value: u24,
}

/// Status register.
#[bitfield(u32, instrospect, forbid_overlaps, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Status {
    /// Indicates that a DMA transfer completed and raised a PI interrupt.
    #[bit(3, rw)]
    interrupt: bool,

    /// TODO test
    #[bit(2, rw)]
    dma_error: bool,

    /// TODO test
    #[bit(1, rw)]
    io_busy: bool,

    /// Indicates that a DMA transfer is in progress.
    #[bit(0, rw)]
    dma_busy: bool,
}

/// Status register, when written to.
#[bitfield(u32, instrospect, forbid_overlaps, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct StatusWrite {
    /// Clears the PI interrupt.
    #[bit(1, rw)]
    clear_interrupt: bool,

    /// Resets the DMA controller.
    /// TODO test
    #[bit(0, rw)]
    reset_dma: bool,
}

// TODO other registers

mapped_registers!(
    START,
    dma_ram_address: DmaRamAddress,
    dma_pi_address: DmaPiAddress,
    dma_read_length: DmaReadLength,
    dma_write_length: DmaWriteLength,
    status: Status,
);
