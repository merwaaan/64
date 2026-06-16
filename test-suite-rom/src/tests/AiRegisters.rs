use alloc::format;
use n64_specs::ai;

use crate::{
    app::App,
    io, no_params, register_test,
    test::{Test, TestError},
};

// This tests records how the AI registers are mirrored over the whole range they're accessible from.
//
// Findings:
// - The 6 registers are mirrored 0x8000 times, every 0x20 bytes, leaving 2 unused slots

register_test!(AiRegistersMirroring);

impl Test for AiRegistersMirroring {
    no_params!();

    fn run(_params: &Self::Params, app: &mut App) -> Result<(), TestError> {
        // Do a DMA to get some values in the registers

        // TODO

        // Read the whole range

        app.memory_region(
            &format!(
                "Read AI registers from {:08X} to {:08X}",
                ai::START,
                ai::END
            ),
            io::uncached_addr(ai::START),
            ai::END - ai::START,
        )?;

        Ok(())
    }
}

// This test records the masking applied to the AI Length register when written to.
//
// This is the only AI register that is both writable and readable.
//
// Findings:
// - TODO

// TODO not really useful as it seems latched, test latching instead?
// TODO test buffering

register_test!(AiLengthRegisterMasking);

impl Test for AiLengthRegisterMasking {
    no_params!();

    fn run(_params: &Self::Params, app: &mut App) -> Result<(), TestError> {
        // Disable DMA to avoid side effects

        io::write_uncached(
            ai::Control::ADDRESS,
            ai::Control::default().with_dma_enabled(false).raw_value(),
        );

        io::write_uncached(ai::DmaLength::ADDRESS, 0x0000_0000u32);

        app.memory(
            "Write 0x0000_0000 to AI DMA length register",
            io::read_uncached(ai::DmaLength::ADDRESS),
        )?;

        io::write_uncached(ai::DmaLength::ADDRESS, 0xFFFF_FFFFu32);

        app.memory(
            "Write 0xFFFF_FFFF to AI DMA length register",
            io::read_uncached(ai::DmaLength::ADDRESS),
        )
    }
}
