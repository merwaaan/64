//! This test records the behavior of the RSP semaphore register.
//!
//! Findings:
//! - The written value is irrelevant, even zero clears the semaphore
//!
//! No surprises:
//! - Reads return the current value and set the register to 1
//! - Writes set the register to 0

#![no_std]
#![no_main]

test_suite_rom::run_test!(RspSemaphoreRegister);

impl Test for RspSemaphoreRegister {
    type Params = u32;

    fn cases() -> Vec<Self::Params> {
        Vec::from([0, 1, 0x1234_5678, 0x8000_0000, 0xFFFF_FFFF])
    }

    fn case_name(value: &u32) -> String {
        format!("Write {:08X}", value)
    }

    fn run(value: &u32, app: &mut App) -> Result<()> {
        let semaphore_reg = specs::rsp::Semaphore::ADDRESS;

        app.push_comment("Clear")?;
        io::write_uncached(semaphore_reg, 0);

        app.push_comment("Read a few times")?;
        app.push_value(io::read_uncached(semaphore_reg))?;
        app.push_value(io::read_uncached(semaphore_reg))?;
        app.push_value(io::read_uncached(semaphore_reg))?;

        app.push_comment("Write the value and read a few times")?;
        io::write_uncached(semaphore_reg, *value);
        app.push_value(io::read_uncached(semaphore_reg))?;
        app.push_value(io::read_uncached(semaphore_reg))?;
        app.push_value(io::read_uncached(semaphore_reg))?;

        app.push_comment("Write the value multiple times before reading again");
        io::write_uncached(semaphore_reg, *value);
        io::write_uncached(semaphore_reg, *value);
        io::write_uncached(semaphore_reg, *value);
        app.push_value(io::read_uncached(semaphore_reg))?;
        app.push_value(io::read_uncached(semaphore_reg))?;
        app.push_value(io::read_uncached(semaphore_reg))?;

        app.push_comment("Write different values before reading again");
        io::write_uncached(semaphore_reg, 0xAAAA_AAAA);
        io::write_uncached(semaphore_reg, *value);
        io::write_uncached(semaphore_reg, 0xBBBB_BBBB);
        app.push_value(io::read_uncached(semaphore_reg))?;
        app.push_value(io::read_uncached(semaphore_reg))?;
        app.push_value(io::read_uncached(semaphore_reg))
    }
}
