//! Interrupts are toggled by setting/clearing specific bits in the MI Mask register.
//! This test records what happens when a write both sets and clears an interrupt.
//!
//! Findings:
//! - Clearing and setting an interrupt mask at the same time does nothing

#![no_std]
#![no_main]

use strum::IntoEnumIterator;

const CLEAR_ALL: u32 = 0x0000_0555;
const SET_ALL: u32 = 0x0000_0AAA;
const UNUSED_BITS: u32 = 0xFFFF_F000;

test_suite_rom::define_test! {
    MiMaskRegisterClearSet {
        type Params = u32;

        fn cases() -> Vec<Self::Params> {
            let mut masks = Vec::with_capacity(6 * 6 + 8);

            // Individual interrupts

            for interrupt in specs::interrupt::Interrupt::iter() {
                masks.push(interrupt.clear_mask());
                masks.push(interrupt.set_mask());
                masks.push(interrupt.set_mask() | interrupt.clear_mask());

                // With unused bits set

                masks.push(UNUSED_BITS | interrupt.clear_mask());
                masks.push(UNUSED_BITS | interrupt.set_mask());
                masks.push(UNUSED_BITS | interrupt.set_mask() | interrupt.clear_mask());
            }

            // All interrupts simultaneously

            masks.push(CLEAR_ALL);
            masks.push(SET_ALL);
            masks.push(CLEAR_ALL | SET_ALL);

            masks.push(UNUSED_BITS | CLEAR_ALL);
            masks.push(UNUSED_BITS | SET_ALL);
            masks.push(UNUSED_BITS | CLEAR_ALL | SET_ALL);

            // Zero

            masks.push(0x0000_0000);
            masks.push(UNUSED_BITS);

            masks
        }

        fn case_name(params: &Self::Params) -> Option<String> {
            Some(format!("{:08X}", *params))
        }

        fn run_case(params: &Self::Params, result: &mut TestCaseResult) {

            let mask_reg = reg_mut_ptr(specs::mi::EnabledInterrupts::ADDRESS);

            result.push_comment("From cleared");

            let value = unsafe {
                mask_reg.write_volatile(CLEAR_ALL);
                mask_reg.write_volatile(*params);
                mask_reg.read_volatile()
            };

            result.push_value(value);

            result.push_comment("From set");

            let value = unsafe {
                mask_reg.write_volatile(SET_ALL);
                mask_reg.write_volatile(*params);
                mask_reg.read_volatile()
            };

            result.push_value(value);
        }
    }
}
