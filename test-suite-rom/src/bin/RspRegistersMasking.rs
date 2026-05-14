//! This test records the masking applied to the RSP registesr when written to.
//!
//! Findings:
//! - The DMA address registers do not read back the written value ??? TODO until DMA starts?
//!
//! No surprises:
//! - The Dma full/busy registers are not writable

#![no_std]
#![no_main]

test_suite_rom::run_test!(RspRegistersMasking);

impl Test for RspRegistersMasking {
    type Params = specs::rsp::Register;

    fn cases() -> Vec<Self::Params> {
        Vec::from([
            specs::rsp::Register::DmaRspAddress,
            specs::rsp::Register::DmaRamAddress,
            //specs::rsp::Register::DmaReadLength, // TODO setup DMA? possible to have empty DMA?
            // specs::rsp::Register::DmaWriteLength,
            specs::rsp::Register::DmaFull,
            specs::rsp::Register::DmaBusy,
        ])

        // TODO PC?
        // TODO test DMA regs?

        // We don't test the Status register as it has different read and write interfaces.
        // We don't test the Semaphore register as it has its own exotic behavior.
    }

    fn case_name(params: &Self::Params) -> String {
        format!("{:?}", *params)
    }

    fn run(reg: &specs::rsp::Register, app: &mut App) -> Result<()> {
        // TODO io uncached helpers
        unsafe {
            let reg_ptr = io::uncached_ptr(reg.address());

            app.comment("Clear")?;
            reg_ptr.write_volatile(0x0000_0000);
            app.value(reg_ptr.read_volatile())?;

            app.comment("Set")?;
            reg_ptr.write_volatile(0xFFFF_FFFF);
            app.value(reg_ptr.read_volatile())?;

            app.comment("Set")?;
            reg_ptr.write_volatile(0x1234_5678);
            app.value(reg_ptr.read_volatile())?;
        };

        Ok(())
    }
}
