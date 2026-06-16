use alloc::format;
use n64_specs::{interrupt, mi};
use strum::IntoEnumIterator;

use crate::{
    app::App,
    io, no_params, register_test,
    test::{Test, TestError},
};

// Interrupts are toggled by setting/clearing specific bits in the MI Mask register.
// This records what happens when a write both sets and clears the same interrupt.
//
// Findings:
// - Clearing and setting an interrupt mask at the same time does nothing

register_test!(MiMaskRegisterClearSet);

const CLEAR_ALL: u32 = 0x0000_0555;
const SET_ALL: u32 = 0x0000_0AAA;
const UNUSED_BITS: u32 = 0xFFFF_F000;

impl Test for MiMaskRegisterClearSet {
    type Params = u32;

    fn cases() -> impl Iterator<Item = Self::Params> {
        interrupt::Interrupt::iter()
            // Individual interrupts
            .flat_map(|interrupt| {
                [
                    interrupt.clear_mask(),
                    interrupt.set_mask(),
                    interrupt.set_mask() | interrupt.clear_mask(),
                    // With unused bits set
                    UNUSED_BITS | interrupt.clear_mask(),
                    UNUSED_BITS | interrupt.set_mask(),
                    UNUSED_BITS | interrupt.set_mask() | interrupt.clear_mask(),
                ]
            })
            .chain([
                // All interrupts simultaneously
                CLEAR_ALL,
                SET_ALL,
                CLEAR_ALL | SET_ALL,
                UNUSED_BITS | CLEAR_ALL,
                UNUSED_BITS | SET_ALL,
                UNUSED_BITS | CLEAR_ALL | SET_ALL,
                // Zero
                0,
                UNUSED_BITS,
            ])
    }

    fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
        let mask_reg = mi::EnabledInterrupts::ADDRESS;

        io::write_uncached(mask_reg, CLEAR_ALL);
        io::write_uncached(mask_reg, *params);

        app.value(
            &format!("Write {:08X} to the cleared MI Mask register", *params),
            io::read_uncached(mask_reg),
        )?;

        io::write_uncached(mask_reg, SET_ALL);
        io::write_uncached(mask_reg, *params);

        app.value(
            &format!("Write {:08X} to the set MI Mask register", *params),
            io::read_uncached(mask_reg),
        )
    }
}

// This test record the value of the MI Version register.
//
// It might be different on different hardware revisions though.

register_test!(MiVersionRegisterValue);

impl Test for MiVersionRegisterValue {
    no_params!();

    fn run(_params: &Self::Params, app: &mut App) -> Result<(), TestError> {
        app.value(
            "MI Version register",
            io::read_uncached(mi::Version::ADDRESS),
        )
    }
}
