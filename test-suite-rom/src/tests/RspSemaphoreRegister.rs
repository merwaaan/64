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
            0x8000_0000,
            0xAAAA_AAAA,
            0xFFFF_FFFF,
        ]
        .into_iter()
    }

    fn run(value: &u32, app: &mut App) -> Result<(), TestError> {
        let semaphore_reg = rsp::Semaphore::ADDRESS;

        // Clear and read a few times

        io::write_uncached(semaphore_reg, 0);

        for i in 0..10 {
            app.value(
                &format!("Read from the semaphore after clearing ({})", i),
                io::read_uncached(semaphore_reg),
            )?;
        }

        // Write and read a few times

        io::write_uncached(semaphore_reg, *value);

        for i in 0..10 {
            app.value(
                &format!(
                    "Read from the semaphore after writing {:08X} ({})",
                    *value, i
                ),
                io::read_uncached(semaphore_reg),
            )?;
        }

        // Write the same value multiple times before reading again

        for _ in 0..10 {
            io::write_uncached(semaphore_reg, *value);
        }

        for i in 0..10 {
            app.value(
                &format!(
                    "Read from the semaphore after writing {:08X} multiple times ({})",
                    *value, i
                ),
                io::read_uncached(semaphore_reg),
            )?;
        }

        // Write different values before reading again

        io::write_uncached(semaphore_reg, 0xABCD_6789u32);
        io::write_uncached(semaphore_reg, *value);
        io::write_uncached(semaphore_reg, 0xBBBB_8787u32);

        for i in 0..10 {
            app.value(
                &format!(
                    "Read from the semaphore after writing various values ({})",
                    i
                ),
                io::read_uncached(semaphore_reg),
            )?;
        }

        Ok(())
    }
}
