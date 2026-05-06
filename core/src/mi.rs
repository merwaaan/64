use arbitrary_int::prelude::*;
use n64_specs as specs;

use crate::{bits::BitTest, cop0::Cop0, location::Location, system::System, value::Value};

pub type MiLocation = Location<{ specs::mi::START }, { specs::mi::END }>;

#[derive(Default, Clone, Copy, Debug)]
pub struct Mi {
    regs: specs::mi::Registers,
}

impl Mi {
    pub fn regs(&self) -> &specs::mi::Registers {
        &self.regs
    }

    pub fn read<T: Value>(s: &System, addr: MiLocation) -> T {
        assert!(T::BYTES == 4, "VI: read with invalid size {}", T::BYTES);

        let offset = addr.relative() & specs::mi::REGISTERS_MASK;

        assert!(
            offset & 3 == 0,
            "VI: read from unaligned address {:08X}",
            offset
        );

        let regs_slice = bytemuck::cast_slice(bytemuck::bytes_of(&s.mi.regs));

        T::read_reg(regs_slice, offset)
    }

    pub fn write<T: Value>(s: &mut System, addr: MiLocation, data: T) {
        assert!(T::BYTES == 4, "MI: write with invalid size {}", T::BYTES);

        let offset = addr.relative() & specs::mi::REGISTERS_MASK;

        assert!(
            offset & 3 == 0,
            "MI: write to unaligned address {:08X}",
            offset
        );

        // Registers are all read-only and writes to MODE and MASK are interpreted as set/clear commands,
        // so we write to a temporary buffer and then update the actual registers from those commands

        let mut fake_reg = [0u32; 1];
        data.write_reg(&mut fake_reg, 0);

        match offset {
            specs::mi::Mode::OFFSET => {
                // TODO what if set/clear both set? samel logic as DP (= does nothing)?

                let command = fake_reg[0];

                if command.bit_is_set::<7>() {
                    s.mi.regs.mode.set_repeat(false);
                } else if command.bit_is_set::<8>() {
                    log::warn!("MI: repeat enabled");
                    s.mi.regs.mode.set_repeat(true);
                }

                if command.bit_is_set::<9>() {
                    s.mi.regs.mode.set_ebus(false);
                } else if command.bit_is_set::<10>() {
                    log::warn!("MI: EBus enabled");
                    s.mi.regs.mode.set_ebus(true);
                }

                if command.bit_is_set::<11>() {
                    s.mi.clear_pending_interrupt(specs::interrupt::Interrupt::Dp, &mut s.cop0);
                    Self::update_cause_register(&s.mi, &mut s.cop0);
                }

                if command.bit_is_set::<12>() {
                    s.mi.regs.mode.set_upper(false);
                } else if command.bit_is_set::<13>() {
                    log::warn!("MI: upper enabled");
                    s.mi.regs.mode.set_upper(true);
                }

                s.mi.regs
                    .mode
                    .set_repeat_count(u7::from_u32(command & 0x7F));
            }

            specs::mi::EnabledInterrupts::OFFSET => {
                let command = fake_reg[0];

                if command.bit_is_set::<0>() {
                    s.mi.regs.enabled_interrupts.set_sp(false);
                } else if command.bit_is_set::<1>() {
                    s.mi.regs.enabled_interrupts.set_sp(true);
                }

                if command.bit_is_set::<2>() {
                    s.mi.regs.enabled_interrupts.set_si(false);
                } else if command.bit_is_set::<3>() {
                    s.mi.regs.enabled_interrupts.set_si(true);
                }

                if command.bit_is_set::<4>() {
                    s.mi.regs.enabled_interrupts.set_ai(false);
                } else if command.bit_is_set::<5>() {
                    s.mi.regs.enabled_interrupts.set_ai(true);
                }

                if command.bit_is_set::<6>() {
                    s.mi.regs.enabled_interrupts.set_vi(false);
                } else if command.bit_is_set::<7>() {
                    s.mi.regs.enabled_interrupts.set_vi(true);
                }

                if command.bit_is_set::<8>() {
                    s.mi.regs.enabled_interrupts.set_pi(false);
                } else if command.bit_is_set::<9>() {
                    s.mi.regs.enabled_interrupts.set_pi(true);
                }

                if command.bit_is_set::<10>() {
                    s.mi.regs.enabled_interrupts.set_dp(false);
                } else if command.bit_is_set::<11>() {
                    s.mi.regs.enabled_interrupts.set_dp(true);
                }

                Self::update_cause_register(&s.mi, &mut s.cop0);
            }

            _ => {}
        }
    }

    fn update_cause_register(mi: &Mi, cop0: &mut Cop0) {
        cop0.set_ip2_interrupt(mi.has_pending_enabled_interrupt());
    }

    pub fn set_pending_interrupt(
        &mut self,
        interrupt: specs::interrupt::Interrupt,
        cop0: &mut Cop0,
    ) {
        self.regs.pending_interrupts = specs::mi::PendingInterrupts::new_with_raw_value(
            self.regs.pending_interrupts.raw_value() | (interrupt as u32),
        );

        Self::update_cause_register(self, cop0);
    }

    pub fn clear_pending_interrupt(
        &mut self,
        interrupt: specs::interrupt::Interrupt,
        cop0: &mut Cop0,
    ) {
        self.regs.pending_interrupts = specs::mi::PendingInterrupts::new_with_raw_value(
            self.regs.pending_interrupts.raw_value() & !(interrupt as u32),
        );

        Self::update_cause_register(self, cop0);
    }

    pub fn is_interrupt_pending(&self, interrupt: specs::interrupt::Interrupt) -> bool {
        self.regs.pending_interrupts.raw_value() & (interrupt as u32) != 0
    }

    pub fn is_interrupt_enabled(&self, interrupt: specs::interrupt::Interrupt) -> bool {
        self.regs.enabled_interrupts.raw_value() & (interrupt as u32) != 0
    }

    pub fn has_pending_enabled_interrupt(&self) -> bool {
        self.regs.pending_interrupts.raw_value() & self.regs.enabled_interrupts.raw_value() != 0
    }
}
