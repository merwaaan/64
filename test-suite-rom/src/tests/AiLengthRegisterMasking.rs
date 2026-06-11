//! This test records the masking applied to the AI Length register when written to.
//!
//! This is the only AI register that is both writable and readable.
//!
//! Findings:
//! - TODO

// TODO not really useful as it seems latched, test latching instead?
// TODO test buffering

use n64_specs::ai;

use crate::{
    app::App,
    io, no_params, register_test,
    test::{Test, TestError},
};

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
