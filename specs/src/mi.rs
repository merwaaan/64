//! MIPS interface, primarily deals with interrupts.
//!
//! TODO
//!
//! https://n64brew.dev/wiki/MIPS_Interface

use arbitrary_int::prelude::*;
use bitbybit::bitfield;

use crate::mapped_registers;

pub const START: u32 = 0x0430_0000;
pub const END: u32 = 0x0440_0000;

pub const REGISTERS_MASK: u32 = 0xF; // TODO check + rename

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Mode {
    /// Upper mode enabled
    #[bit(9, rw)]
    upper: bool,

    /// EBus mode enabled
    #[bit(8, rw)]
    ebus: bool,

    /// Repeat mode enabled
    #[bit(7, rw)]
    repeat: bool,

    /// Bytes to write in repeat mode, minus 1
    #[bits(0..=6, rw)]
    repeat_count: u7,
}

// TODO set here? or in user code?
const VERSION_DEFAULT: u32 = 0x02020102;

#[bitfield(u32, forbid_overlaps, instrospect, default = VERSION_DEFAULT, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Version {
    #[bits(24..=31, rw)]
    rsp: u8,

    #[bits(16..=23, rw)]
    rdp: u8,

    #[bits(8..=15, rw)]
    rac: u8,

    #[bits(0..=7, rw)]
    io: u8,
}

macro_rules! interrupts_reg {
    ($name:ident) => {
        #[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
        #[derive(bytemuck::Pod, bytemuck::Zeroable)]
        pub struct $name {
            #[bit(5, rw)]
            dp: bool,

            #[bit(4, rw)]
            pi: bool,

            #[bit(3, rw)]
            vi: bool,

            #[bit(2, rw)]
            ai: bool,

            #[bit(1, rw)]
            si: bool,

            #[bit(0, rw)]
            sp: bool,
        }
    };
}

interrupts_reg!(PendingInterrupts);

interrupts_reg!(EnabledInterrupts);

mapped_registers!(START, mode: Mode, version: Version, pending_interrupts: PendingInterrupts, enabled_interrupts: EnabledInterrupts);
