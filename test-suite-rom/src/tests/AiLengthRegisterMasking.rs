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
        // Disable DMA

        io::write_uncached(
            ai::Control::ADDRESS,
            ai::Control::default().with_dma_enabled(false).raw_value(),
        );

        let length_reg = io::uncached_ptr::<u32>(ai::DmaLength::ADDRESS);

        unsafe {
            length_reg.write_volatile(0x0000_0000); // TODO write_uncached
            app.memory(
                "Write 0x0000_0000 to AI DMA length register",
                length_reg as u32,
            )?;

            length_reg.write_volatile(0xFFFF_FFFF); // TODO write_uncached
            app.memory(
                "Write 0xFFFF_FFFF to AI DMA length register",
                length_reg as u32,
            )?;
        }

        Ok(())
    }
}
