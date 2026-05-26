//! Records how the PI registers are mirrored over the whole range they're accessible from.
//!
//! Findings:
//! - the registers are mirrored 0x4000 times, every 0x40 bytes
//! - after BSD_DOM2_RLS at 0x30-0x33, there is a 12-bytes gap, enough space for three registers
//!
//! Unclear:
//! - reading from those unused slots returns different values other the whole range
//!   - 0x34 and 0x38 read the same value: 0xFF00ABCD in the first 2048 mirrors and then 0 until the end
//!   - 0x3C reads 7 // TODO mirror of PI_BSD_DOM1_PGS? or coincidence?

// TODO 0xFF00ABCD is actually the value i write to AUX?! for 2048 mirrors because of partial decoder width?

use n64_specs::pi;

use crate::{
    app::App,
    io, no_params, register_test,
    test::{Test, TestError},
};

register_test!(PiRegistersMirroring);

impl Test for PiRegistersMirroring {
    no_params!();

    fn run(_params: &Self::Params, app: &mut App) -> Result<(), TestError> {
        // TODO DMA to init values?

        // Read the whole range of PI registers

        app.memory_region(io::uncached_addr(pi::START), pi::END - pi::START)?;

        Ok(())
    }
}
