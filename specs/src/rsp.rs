//! Reality Signal Processor
//!
//! This is a slimmed down version of the main MIPS processor victor vector instructions on top:
//! - Registers are strictly 32-bit
//! - No exceptions or traps
//! - Less arithmetic instructions (no mult/div, no 64-bit instructions like DADD/DSUB)
//! - Cannot access RAM directly, transfers it to/from DMEM using DMA instead
//! - The PC is 12-bit and wraps around IMEM
//!
//! TODO COP 0 = SP + DP registers
//!
//! TODO vector! = COP 2
//!
//! Resources:
//! - Nintendo Ultra64 RSP Programmer’s Guide https://ultra64.ca/files/documentation/silicon-graphics/SGI_Nintendo_64_RSP_Programmers_Guide.pdf
//! - N64brew / Reality Signal Processor https://n64brew.dev/wiki/Reality_Signal_Processor

use arbitrary_int::prelude::*;
use bitbybit::bitfield;

use crate::mapped_registers;

pub const MEMORY_START: u32 = 0x0400_0000;
pub const MEMORY_END: u32 = 0x0404_0000;
pub const MEMORY_MASK: u32 = 0x1FFF; // TODO what for?
pub const MEMORY_BANK_SIZE: u32 = 0x1000;

pub const REGISTERS_START: u32 = MEMORY_END;
pub const REGISTERS_END: u32 = 0x040C_0000;
pub const REGISTERS_MASK: u32 = 0x1F;

/// Address in the RSP memory to transfer data from/to via DMA.
#[bitfield(u32, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct DmaRspAddress {
    /// Selected bank (0 = DMEM, 1 = IMEM).
    #[bit(12, rw)]
    bank: bool,

    /// Offset in the selected bank.
    #[bits(0..=11, rw)]
    offset: u12,

    /// Full address, including bank and offset.
    /// TODO bit 0 3 writable?
    #[bits(0..=12, rw)]
    value: u13,
}

/// Address in the main RAM to transfer data from/to via DMA.
#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct DmaRamAddress {
    /// TODO bit 0 3 writable?
    #[bits(0..=23, rw)]
    value: u24,
}

macro_rules! dma_length_reg {
    ($(#[$attrs:meta])* $name:ident) => {
        $(#[$attrs])*
        #[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
        #[derive(bytemuck::Pod, bytemuck::Zeroable)]
        pub struct $name {
            /// Number of bytes to skip after each row.
            ///
            /// Only applies to the data read from/written to RAM!
            /// TODO bit 0 3 writable?
            #[bits(20..=31, rw)]
            skip: u12,

            /// Number of rows to transfer, minus 1.
            #[bits(12..=19, rw)]
            rows: u8,

            /// Length of the transfer in bytes, minus 1.
            /// TODO bit 0 3 writable?
            #[bits(0..=11, rw)]
            length: u12,
        }
    };
}

// TODO helpers

dma_length_reg! {
    /// Layout of the data to be transferred from RAM to RSP memory.
    /// Initiates the DMA transfer on writes.
    DmaReadLength
}

dma_length_reg!(
    /// Layout of the data to be transferred from RSP memory to RAM.
    /// Initiates the DMA transfer on writes.
    DmaWriteLength
);

/// Main control register.
#[bitfield(u32, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Status {
    /// Signal 7, for communicating between CPU and RSP.
    #[bit(14, rw)]
    sig7: bool,

    /// Signal 6.
    #[bit(13, rw)]
    sig6: bool,

    /// Signal 5.
    #[bit(12, rw)]
    sig5: bool,

    /// Signal 4.
    #[bit(11, rw)]
    sig4: bool,

    /// Signal 3.
    #[bit(10, rw)]
    sig3: bool,

    /// Signal 2.
    #[bit(9, rw)]
    sig2: bool,

    /// Signal 1.
    #[bit(8, rw)]
    sig1: bool,

    /// Signal 0.
    #[bit(7, rw)]
    sig0: bool,

    /// Generates an MI interrupt when the RSP hits a BREAK instruction.
    #[bit(6, rw)]
    interrupt_on_break: bool,

    /// Single-step mode.
    /// TODO test
    #[bit(5, rw)]
    single_step: bool,

    /// TODO?
    #[bit(4, rw)]
    io_busy: bool,

    /// Indicates that the DMA queue is full (ie. there's a DMA in progress and another one pending).
    /// Mirrors the Dma full register.
    #[bit(3, rw)]
    dma_full: bool,

    /// Indicates that there's a DMA transfer in progress.
    /// Mirrors the Dma busy register.
    #[bit(2, rw)]
    dma_busy: bool,

    /// Set when the RSP hit a BREAK instruction. // TODO does BREAK halt?
    #[bit(1, rw)]
    broke: bool,

    /// Running state of the RSP.
    #[bit(0, rw)]
    halted: bool,
}

/// Status register when written to.
#[bitfield(u32, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct StatusWrite {
    #[bit(24, rw)]
    set_sig7: bool,

    #[bit(23, rw)]
    clear_sig7: bool,

    #[bit(22, rw)]
    set_sig6: bool,

    #[bit(21, rw)]
    clear_sig6: bool,

    #[bit(20, rw)]
    set_sig5: bool,

    #[bit(19, rw)]
    clear_sig5: bool,

    #[bit(18, rw)]
    set_sig4: bool,

    #[bit(17, rw)]
    clear_sig4: bool,

    #[bit(16, rw)]
    set_sig3: bool,

    #[bit(15, rw)]
    clear_sig3: bool,

    #[bit(14, rw)]
    set_sig2: bool,

    #[bit(13, rw)]
    clear_sig2: bool,

    #[bit(12, rw)]
    set_sig1: bool,

    #[bit(11, rw)]
    clear_sig1: bool,

    #[bit(10, rw)]
    set_sig0: bool,

    #[bit(9, rw)]
    clear_sig0: bool,

    #[bit(8, rw)]
    set_interrupt_on_break: bool,

    #[bit(7, rw)]
    clear_interrupt_on_break: bool,

    #[bit(6, rw)]
    set_single_step: bool,

    #[bit(5, rw)]
    clear_single_step: bool,

    #[bit(4, rw)]
    set_interrupt: bool,

    #[bit(3, rw)]
    clear_interrupt: bool,

    #[bit(2, rw)]
    clear_broke: bool,

    #[bit(1, rw)]
    set_halt: bool,

    #[bit(0, rw)]
    clear_halt: bool,
}

/// Indicates that the DMA queue is full (ie. there's a DMA in progress and another one pending).
/// Mirrors the corresponding bit in the Status register.
#[bitfield(u32, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct DmaFull {
    #[bit(0, rw)]
    value: bool,
}

/// Indicates that there's a DMA transfer in progress.
/// Mirrors the corresponding bit in the Status register.
#[bitfield(u32, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct DmaBusy {
    #[bit(0, rw)]
    value: bool,
}

/// Semaphore for synchronizing CPU and RSP programs.
/// Reads return the current value and automatically set the register to 1 for future reads.
/// So by writing 0, CPU/RSP programs can signal to the other side that the semaphore is now free to use.
#[bitfield(u32, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Semaphore {
    #[bit(0, rw)]
    value: bool,
}

/// Program counter, relative to IMEM.
#[bitfield(u32, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct ProgramCounter {
    #[bits(0..=11, rw)]
    value: u12,
}

mapped_registers!(
    REGISTERS_START,
    dma_rsp_address: DmaRspAddress,
    dma_ram_address: DmaRamAddress,
    dma_read_length: DmaReadLength,
    dma_write_length: DmaWriteLength,
    status: Status,
    dma_full: DmaFull,
    dma_busy: DmaBusy,
    semaphore: Semaphore
);

impl Registers {
    /// Sets both Dma busy mirrors
    pub fn set_dma_busy(&mut self, busy: bool) {
        self.dma_busy.set_value(busy);
        self.status.set_dma_busy(busy);
    }

    /// Sets both Dma full mirrors
    pub fn set_dma_full(&mut self, full: bool) {
        self.dma_full.set_value(full);
        self.status.set_dma_full(full);
    }
}
