use alloc::{format, vec::Vec};
use arbitrary_int::u24;

use crate::{
    app::App,
    io, register_test,
    test::{Test, TestError},
};

// Dummy test to validate the recording mechanism and various helpers.

register_test!(Dummy);

impl Test for Dummy {
    type Params = bool;

    fn cases() -> impl Iterator<Item = Self::Params> {
        [true, false].into_iter()
    }

    fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
        app.value(
            "Value derived from the parameter",
            if *params { u32::MAX } else { 0 },
        )?;

        let ram_data = (0..1000).map(|i| i as u32).collect::<Vec<_>>();

        for i in 0..10 {
            app.memory(&format!("byte {} of test buffer", i), unsafe {
                ram_data.as_ptr().add(i) as u32
            })?;
        }

        app.memory_region(
            "Whole test buffer",
            ram_data.as_ptr() as u32,
            ram_data.len() as u32 * 4,
        )?;

        //

        let ram_data =
            io::UncachedBuffer::<u8>::with_alignment(0x40, n64_specs::pi::DMA_RAM_ALIGNMENT);

        io::pi_dma(
            &io::PiDma {
                direction: io::PiDmaDirection::PiToRam,
                ram_address: u24::from_u32(io::physical_addr(ram_data.as_ptr() as u32)),
                pi_address: 0x1000_0000,
                length: u24::from_u8(0x40 - 1),
            },
            true,
        );

        app.memory_region("PI DMA", ram_data.as_ptr() as u32, ram_data.len() as u32)?;

        //

        // TODO app.comment("Test PI DMA")?;

        Ok(())
    }
}
