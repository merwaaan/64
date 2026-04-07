use arbitrary_int::prelude::*;
use bitbybit::bitfield;
use strum::{Display, EnumIter};

use crate::{
    bits::BitTest, cop0::Cop0, location::Location, register_overlaps, system::System, value::Value,
};

/// MIPS interface, primarily deals with interrupts.
///
/// https://n64brew.dev/wiki/MIPS_Interface

#[derive(Debug, Clone, Copy, Display, EnumIter)]
#[repr(u8)]
pub enum Interrupt {
    Sp = 1,
    Si = 1 << 1,
    Ai = 1 << 2,
    Vi = 1 << 3,
    Pi = 1 << 4,
    Dp = 1 << 5,
}

pub type MiLocation = Location<0x0430_0000, 0x0440_0000>;

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Mode {
    /// Bytes to write in repeat mode, minus 1
    #[bits(0..=6, rw)]
    repeat_count: u7,

    /// Repeat mode enabled
    #[bit(7, rw)]
    repeat: bool,

    /// EBus mode enabled
    #[bit(8, rw)]
    ebus: bool,

    /// Upper mode enabled
    #[bit(9, rw)]
    upper: bool,
}

const VERSION_DEFAULT: u32 = 0x02020102;

#[bitfield(u32, forbid_overlaps, instrospect, default = VERSION_DEFAULT, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Version {
    #[bits(0..=7, rw)]
    io: u8,

    #[bits(8..=15, rw)]
    rac: u8,

    #[bits(16..=23, rw)]
    rdp: u8,

    #[bits(24..=31, rw)]
    rsp: u8,
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Interrupts {
    #[bit(0, rw)]
    sp: bool,

    #[bit(1, rw)]
    si: bool,

    #[bit(2, rw)]
    ai: bool,

    #[bit(3, rw)]
    vi: bool,

    #[bit(4, rw)]
    pi: bool,

    #[bit(5, rw)]
    dp: bool,
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Registers {
    pub mode: Mode,
    pub version: Version,
    pub interrupts: Interrupts,
    pub mask: Interrupts,
}

const REGISTERS_MASK: u32 = 0xF;

#[derive(Default, Clone, Copy, Debug)]
pub struct Mi {
    regs: Registers,
}

struct WriteResult {
    check_interrupts: bool,
    clear_dp_interrupt: bool,
}

impl Registers {
    fn read<T: Value>(&self, offset: u32) -> T {
        let words = bytemuck::cast_slice(bytemuck::bytes_of(self));

        T::read_reg(words, offset)
    }

    fn write<T: Value>(&mut self, offset: u32, data: T) -> WriteResult {
        let mut result = WriteResult {
            check_interrupts: false,
            clear_dp_interrupt: false,
        };

        // Registers are all read-only and writes to MODE and MASK are interpreted as set/clear commands,
        // so we write to a temporary buffer and then update the actual registers from those commands

        let mut fake_regs = [0u32; 4];
        data.write_reg(&mut fake_regs, offset);

        // Mode

        if register_overlaps!(offset, offset + T::BYTES as u32, Registers::mode) {
            // TODO what if set/clear both set? samel logic as DP (= does nothing)?

            let mode_command = fake_regs[0];

            let mut mode = self.mode;

            if mode_command.bit_is_set::<7>() {
                mode.set_repeat(false);
            } else if mode_command.bit_is_set::<8>() {
                log::warn!("MI: repeat enabled");
                mode.set_repeat(true);
            }

            if mode_command.bit_is_set::<9>() {
                mode.set_ebus(false);
            } else if mode_command.bit_is_set::<10>() {
                log::warn!("MI: EBus enabled");
                mode.set_ebus(true);
            }

            if mode_command.bit_is_set::<11>() {
                result.clear_dp_interrupt = true;
            }

            if mode_command.bit_is_set::<12>() {
                mode.set_upper(false);
            } else if mode_command.bit_is_set::<13>() {
                log::warn!("MI: upper enabled");
                mode.set_upper(true);
            }

            mode.set_repeat_count(u7::from_u32(mode_command & 0x7F));

            self.mode = mode;
        }

        // Mask

        if register_overlaps!(offset, offset + T::BYTES as u32, Registers::mask) {
            let mask_command = fake_regs[3];

            let mut mask = self.mask;

            if mask_command.bit_is_set::<0>() {
                mask.set_sp(false);
            } else if mask_command.bit_is_set::<1>() {
                mask.set_sp(true);
            }

            if mask_command.bit_is_set::<2>() {
                mask.set_si(false);
            } else if mask_command.bit_is_set::<3>() {
                mask.set_si(true);
            }

            if mask_command.bit_is_set::<4>() {
                mask.set_ai(false);
            } else if mask_command.bit_is_set::<5>() {
                mask.set_ai(true);
            }

            if mask_command.bit_is_set::<6>() {
                mask.set_vi(false);
            } else if mask_command.bit_is_set::<7>() {
                mask.set_vi(true);
            }

            if mask_command.bit_is_set::<8>() {
                mask.set_pi(false);
            } else if mask_command.bit_is_set::<9>() {
                mask.set_pi(true);
            }

            if mask_command.bit_is_set::<10>() {
                mask.set_dp(false);
            } else if mask_command.bit_is_set::<11>() {
                mask.set_dp(true);
            }

            self.mask = mask;

            result.check_interrupts = true;
        }

        result
    }
}

impl Mi {
    pub fn regs(&self) -> &Registers {
        &self.regs
    }

    pub fn read<T: Value>(s: &System, addr: MiLocation) -> T {
        s.mi.regs.read(addr.relative() & REGISTERS_MASK)
    }

    pub fn write<T: Value>(s: &mut System, addr: MiLocation, data: T) {
        let offset = addr.relative() & REGISTERS_MASK;

        let mut result = s.mi.regs.write(offset, data);

        if result.clear_dp_interrupt {
            s.mi.clear_pending_interrupt(Interrupt::Dp, &mut s.cop0);
            result.check_interrupts = true;
        }

        if result.check_interrupts {
            Self::update_cause_register(&s.mi, &mut s.cop0);
        }
    }

    fn update_cause_register(mi: &Mi, cop0: &mut Cop0) {
        cop0.set_ip2_interrupt(mi.has_pending_enabled_interrupt());
    }

    pub fn set_pending_interrupt(&mut self, interrupt: Interrupt, cop0: &mut Cop0) {
        self.regs.interrupts =
            Interrupts::new_with_raw_value(self.regs.interrupts.raw_value | (interrupt as u32));

        Self::update_cause_register(self, cop0);
    }

    pub fn clear_pending_interrupt(&mut self, interrupt: Interrupt, cop0: &mut Cop0) {
        self.regs.interrupts =
            Interrupts::new_with_raw_value(self.regs.interrupts.raw_value & !(interrupt as u32));

        Self::update_cause_register(self, cop0);
    }

    pub fn is_interrupt_pending(&self, interrupt: Interrupt) -> bool {
        self.regs.interrupts.raw_value & (interrupt as u32) != 0
    }

    pub fn is_interrupt_enabled(&self, interrupt: Interrupt) -> bool {
        self.regs.mask.raw_value & (interrupt as u32) != 0
    }

    pub fn has_pending_enabled_interrupt(&self) -> bool {
        self.regs.interrupts.raw_value & self.regs.mask.raw_value != 0
    }
}
