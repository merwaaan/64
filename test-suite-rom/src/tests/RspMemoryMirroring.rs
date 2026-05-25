//! Records how the RSP DMEM/IMEM is mirrored over the whole range it's accessible from.
//!
//! No surprises:
//! - the memory is mirrored 32 times, every 0x2000 bytes, without unexpected patterns

use n64_specs::rsp;

use crate::{
    app::App,
    io, no_params, register_test,
    test::{Test, TestError},
};

register_test!(RspMemoryMirroring);

impl Test for RspMemoryMirroring {
    no_params!();

    fn run(_params: &Self::Params, app: &mut App) -> Result<(), TestError> {
        // Fill DMEM and IMEM

        let mem = io::uncached_ptr(rsp::MEMORY_START);

        for i in (0..rsp::DMEM_SIZE + rsp::IMEM_SIZE).step_by(4) {
            io::write_uncached(mem as u32 + i, i);
        }

        // Read the whole memory range

        app.memory_region(
            io::uncached_ptr(rsp::MEMORY_START) as u32,
            rsp::MEMORY_END - rsp::MEMORY_START,
        )?;

        Ok(())
    }
}
