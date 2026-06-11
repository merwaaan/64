//! Records the RSP registers masking when written to.
//!
//! Findings:
//! - The DMA address registers do not read back the written value ??? TODO until DMA starts?
//!
//! No surprises:
//! - The Dma full/busy registers are not writable

use alloc::format;
use n64_specs::rsp;

use crate::{
    app::App,
    io, register_test,
    test::{Test, TestError},
};

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
