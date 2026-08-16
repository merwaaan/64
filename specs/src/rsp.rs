//! Reality Signal Processor
//!
//! This is a slimmed down version of the main MIPS processor with vector instructions on top:
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

pub mod instructions;
pub mod registers;

pub const MEMORY_START: u32 = 0x0400_0000;
pub const MEMORY_END: u32 = 0x0404_0000; // TODO DMEM + IMEM = 0x2000, is it mirrored???
pub const MEMORY_MASK: u32 = 0x1FFF; // TODO what for?
pub const MEMORY_BANK_SIZE: u32 = 0x1000;

pub const DMEM_START: u32 = MEMORY_START;
pub const DMEM_SIZE: u32 = MEMORY_BANK_SIZE;
pub const DMEM_END: u32 = DMEM_START + DMEM_SIZE;

pub const IMEM_START: u32 = DMEM_END;
pub const IMEM_SIZE: u32 = MEMORY_BANK_SIZE;
pub const IMEM_END: u32 = IMEM_START + IMEM_SIZE;

pub const REGISTERS_START: u32 = MEMORY_END;
pub const REGISTERS_END: u32 = 0x040C_0000;
pub const REGISTERS_MASK: u32 = 0x1F;

// TODO const DMA alignments?
