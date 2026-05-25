//! Records the behavior of the RSP semaphore register.
//!
//! Findings:
//! - The written value is irrelevant, even zero clears the semaphore
//!
//! No surprises:
//! - Reads return the current value and set the register to 1
//! - Writes set the register to 0

use alloc::format;
use n64_specs::rsp;

use crate::{
    app::App,
    io, register_test,
    test::{Test, TestError},
};

register_test!(RspSemaphoreRegister);

impl Test for RspSemaphoreRegister {
    type Params = u32;

    fn cases() -> impl Iterator<Item = Self::Params> {
        [
            0,
            1,
            0x1234_5678,
            0x8000_0000,
            0x5555_5555,
            0xAAAA_AAAA,
            0xFFFF_FFFF,
        ]
        .into_iter()
    }

    fn run(value: &u32, app: &mut App) -> Result<(), TestError> {
        app.comment(&format!(
            "Write {:08X} to the RSP semaphore register",
            value
        ))?;

        let semaphore_reg = rsp::Semaphore::ADDRESS;

        app.comment("Clear")?;
        io::write_uncached(semaphore_reg, 0);

        app.comment("Read a few times")?;
        app.value(io::read_uncached(semaphore_reg))?;
        app.value(io::read_uncached(semaphore_reg))?;
        app.value(io::read_uncached(semaphore_reg))?;

        app.comment("Write the value and read a few times")?;
        io::write_uncached(semaphore_reg, *value);
        app.value(io::read_uncached(semaphore_reg))?;
        app.value(io::read_uncached(semaphore_reg))?;
        app.value(io::read_uncached(semaphore_reg))?;

        app.comment("Write the value multiple times before reading again");
        io::write_uncached(semaphore_reg, *value);
        io::write_uncached(semaphore_reg, *value);
        io::write_uncached(semaphore_reg, *value);
        app.value(io::read_uncached(semaphore_reg))?;
        app.value(io::read_uncached(semaphore_reg))?;
        app.value(io::read_uncached(semaphore_reg))?;

        app.comment("Write different values before reading again");
        io::write_uncached(semaphore_reg, 0xABCD_6789);
        io::write_uncached(semaphore_reg, *value);
        io::write_uncached(semaphore_reg, 0xBBBB_8787);
        app.value(io::read_uncached(semaphore_reg))?;
        app.value(io::read_uncached(semaphore_reg))?;
        app.value(io::read_uncached(semaphore_reg))
    }
}
