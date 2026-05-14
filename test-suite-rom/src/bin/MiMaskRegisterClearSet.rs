//! Interrupts are toggled by setting/clearing specific bits in the MI Mask register.
//! This test records what happens when a write both sets and clears an interrupt.
//!
//! Findings:
//! - Clearing and setting an interrupt mask at the same time does nothing

#![no_std]
#![no_main]

use strum::{EnumCount, IntoEnumIterator};

const CLEAR_ALL: u32 = 0x0000_0555;
const SET_ALL: u32 = 0x0000_0AAA;
const UNUSED_BITS: u32 = 0xFFFF_F000;

test_suite_rom::run_test!(MiMaskRegisterClearSet);

impl Test for MiMaskRegisterClearSet {
    type Params = u32;

    fn cases() -> Vec<Self::Params> {
        let mut masks = Vec::with_capacity(specs::interrupt::Interrupt::COUNT * 6 + 8);

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

    fn case_name(params: &Self::Params) -> String {
        format!("{:08X}", *params)
    }

    fn run(params: &Self::Params, app: &mut App) -> Result<()> {
        let mask_reg = specs::mi::EnabledInterrupts::ADDRESS;

        app.push_comment("From cleared")?;
        io::write_uncached(mask_reg, CLEAR_ALL);
        io::write_uncached(mask_reg, *params);
        app.push_value(io::read_uncached(mask_reg))?;

        app.push_comment("From set")?;
        io::write_uncached(mask_reg, SET_ALL);
        io::write_uncached(mask_reg, *params);
        app.push_value(io::read_uncached(mask_reg))
    }
}
