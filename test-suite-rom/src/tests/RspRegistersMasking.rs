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
        // TODO io uncached helpers
        unsafe {
            let reg_ptr = io::uncached_ptr::<u32>(reg.address());

            reg_ptr.write_volatile(0x0000_0000);
            app.value(
                &format!("Read from RSP {} register after clearing all bits", reg),
                reg_ptr.read_volatile(),
            )?;

            reg_ptr.write_volatile(0xFFFF_FFFF);
            app.value(
                &format!("Read from RSP {} register after setting all bits", reg),
                reg_ptr.read_volatile(),
            )?;
        };

        Ok(())
    }
}
