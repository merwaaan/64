//! Records how the RSP DMEM/IMEM is mirrored over the whole range it's accessible from.
//!
//! No surprises:
//! - the memory is mirrored 32 times, every 0x2000 bytes, without unexpected patterns

use alloc::format;
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
        // Fill DMEM and IMEM and then read the whole memory range

        let mem_start = io::uncached_addr(rsp::MEMORY_START);

        for i in (0..rsp::DMEM_SIZE + rsp::IMEM_SIZE).step_by(4) {
            io::write_uncached(mem_start + i, i);
        }

        app.memory_region(
            &format!(
                "Read RSP memory from {:08X} to {:08X}",
                rsp::MEMORY_START,
                rsp::MEMORY_END
            ),
            mem_start,
            rsp::MEMORY_END - rsp::MEMORY_START,
        )?;

        // Write to each successive 0x2000 region and read back the base region

        for mirror in 1..31 {
            let mirror_start = mem_start + mirror * 0x2000;

            for i in (0..rsp::DMEM_SIZE + rsp::IMEM_SIZE).step_by(4) {
                io::write_uncached(mirror_start + i, mirror);
            }

            app.memory_region(
                &format!(
                    "Write to mirror {} ({:08X} to {:08X}) and read back from {:08X} to {:08X}",
                    mirror,
                    mirror_start,
                    mirror_start + rsp::DMEM_SIZE + rsp::IMEM_SIZE,
                    rsp::MEMORY_START,
                    rsp::MEMORY_START + rsp::DMEM_SIZE + rsp::IMEM_SIZE
                ),
                mem_start,
                rsp::DMEM_SIZE + rsp::IMEM_SIZE,
            )?;
        }

        Ok(())
    }
}
