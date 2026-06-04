//! This tests records how the AI registers are mirrored over the whole range they're accessible from.
//!
//! Findings:
//! - The 6 registers are mirrored 0x8000 times, every 0x20 bytes, leaving 2 unused slots

use alloc::format;
use n64_specs::ai;

use crate::{
    app::App,
    io, no_params, register_test,
    test::{Test, TestError},
};

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
