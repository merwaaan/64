//! This tests records how the AI registers are mirrored over the whole range they're accessible from.
//!
//! Findings:
//! - The 6 registers are mirrored every 8 words
//! - Writing to the 2 unused slots has no effect

use alloc::format;
use n64_specs::ai;

use crate::{
    app::App,
    io, no_params,
    test::{Test, TestError},
};

pub struct AiRegistersMirroring;

impl Test for AiRegistersMirroring {
    no_params!();

    fn run(_params: &Self::Params, app: &mut App) -> Result<(), TestError> {
        app.comment(format!("Read from {:08X} to {:08X}", ai::START, ai::END).as_str())?;

        app.memory_region(io::uncached_ptr(ai::START) as u32, ai::END - ai::START)?;

        for reg in [6, 7] {
            for value in [0, u32::MAX] {
                app.comment(format!("Write {} to unused slot #{}", value, reg).as_str())?;

                io::write_uncached(ai::START + reg * 4, value);

                app.memory_region(io::uncached_ptr(ai::START) as u32, 8 * 4)?;
            }
        }

        Ok(())
    }
}
