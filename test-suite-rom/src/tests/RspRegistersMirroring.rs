//! Records how the RSP registers are mirrored over the whole range they're accessible from.
//!
//! No surprises:
//! - the registers are mirrored every 8 words without gaps or unexpected patterns

use n64_specs::rsp;

use crate::{
    app::App,
    io, no_params, register_test,
    test::{Test, TestError},
};

register_test!(RspRegistersMirroring);

impl Test for RspRegistersMirroring {
    no_params!();

    fn run(_params: &Self::Params, app: &mut App) -> Result<(), TestError> {
        // Give known values to the registers when possible to make them a bit more recognizable in the output

        // TODO no readback of addr regs?

        io::write_uncached(
            rsp::Status::ADDRESS,
            rsp::StatusWrite::default()
                .with_clear_sig7(true)
                .with_set_sig6(true)
                .with_clear_sig5(true)
                .with_set_sig4(true)
                .with_clear_sig3(true)
                .with_set_sig2(true)
                .with_clear_sig1(true)
                .with_set_sig0(true)
                .with_set_interrupt_on_break(true)
                .with_clear_single_step(true)
                .with_set_halt(true)
                .raw_value(),
        );

        // TODO DMA to set addr regs

        io::read_uncached(rsp::Semaphore::ADDRESS); // Switch the semaphore to 1

        // Read the whole range of RSP registers

        app.memory_region(
            io::uncached_ptr(rsp::REGISTERS_START) as u32,
            rsp::REGISTERS_END - rsp::REGISTERS_START,
        )?;

        Ok(())
    }
}
