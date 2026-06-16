use alloc::format;
use n64_specs::rsp;

use crate::{
    app::App,
    io, no_params, register_test,
    test::{Test, TestError},
};

// Records the RSP registers masking when written to.
//
// Findings:
// - The DMA address registers do not read back the written value ??? TODO until DMA starts?
//
// No surprises:
// - The Dma full/busy registers are not writable

register_test!(RspRegistersMasking);

impl Test for RspRegistersMasking {
    type Params = rsp::Register;

    fn cases() -> impl Iterator<Item = Self::Params> {
        [
            rsp::Register::DmaRspAddress,
            rsp::Register::DmaRamAddress,
            // rsp::Register::DmaReadLength, // TODO setup DMA? possible to have empty DMA?
            // rsp::Register::DmaWriteLength,
            rsp::Register::DmaFull,
            rsp::Register::DmaBusy,
        ]
        .into_iter()

        // TODO PC?
        // TODO test DMA regs?

        // We don't test the Status register as it has different read and write interfaces.
        // We don't test the Semaphore register as it has its own exotic behavior.
    }

    fn run(reg: &rsp::Register, app: &mut App) -> Result<(), TestError> {
        io::write_uncached(reg.address(), 0x0000_0000u32);

        app.value(
            &format!("Read from RSP {} register after clearing all bits", reg),
            io::read_uncached(reg.address()),
        )?;

        io::write_uncached(reg.address(), 0xFFFF_FFFFu32);

        app.value(
            &format!("Read from RSP {} register after setting all bits", reg),
            io::read_uncached(reg.address()),
        )
    }
}

// Records how the RSP registers are mirrored over the whole range they're accessible from.
//
// No surprises:
// - the registers are mirrored every 8 words without gaps or unexpected patterns

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

        io::read_uncached::<u32>(rsp::Semaphore::ADDRESS); // Switch the semaphore to 1

        // Read the whole range

        app.memory_region(
            &format!(
                "Read RSP registersfrom {:08X} to {:08X}",
                rsp::REGISTERS_START,
                rsp::REGISTERS_END
            ),
            io::uncached_addr(rsp::REGISTERS_START),
            rsp::REGISTERS_END - rsp::REGISTERS_START,
        )?;

        Ok(())
    }
}

// Records the behavior of the RSP semaphore register.
//
// Findings:
// - The written value is irrelevant, even zero clears the semaphore
//
// No surprises:
// - Reads return the current value and set the register to 1
// - Writes set the register to 0

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
