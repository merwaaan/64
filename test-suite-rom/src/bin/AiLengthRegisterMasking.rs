//! This test records the masking applied to the AI Length register when written to.
//!
//! This is the only AI register that is both writable and readable.
//!
//! Findings:
//! - TODO

// TODO not really useful as it seems latched, test latching instead?
// TODO test buffering

#![no_std]
#![no_main]

test_suite_rom::run_test!(AiLengthRegisterMasking);

impl Test for AiLengthRegisterMasking {
    no_params!();

    fn run(_params: &Self::Params, app: &mut App) -> Result<(), TestError> {
        // Disable DMA

        io::write_uncached(
            specs::ai::Control::ADDRESS,
            specs::ai::Control::default()
                .with_dma_enabled(false)
                .raw_value(),
        );

        let length_reg = io::uncached_ptr(specs::ai::DmaLength::ADDRESS);

        unsafe {
            app.comment("Write 0x0000_0000 to the AI DMA length register")?;
            length_reg.write_volatile(0x0000_0000); // TODO write_uncached
            app.memory(length_reg as u32)?;

            app.comment("Write 0xFFFF_FFFF to the AI DMA length register")?;
            length_reg.write_volatile(0xFFFF_FFFF); // TODO write_uncached
            app.memory(length_reg as u32)?;
        }

        Ok(())
    }
}
