//! Dummy test to validate the recording mechanism and various helpers.

use alloc::{format, vec::Vec};
use arbitrary_int::u24;

use crate::{
    app::App,
    io,
    test::{Test, TestError},
};

pub struct Dummy;

impl Test for Dummy {
    type Params = bool;

    fn cases() -> impl Iterator<Item = Self::Params> {
        [true, false].into_iter()
    }

    fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
        app.comment(&format!("Dummy test with params {:?}", params))?;

        app.value(if *params { u32::MAX } else { 0 })?;

        let ram_data = (0..1000).map(|i| i as u32).collect::<Vec<_>>();

        for i in 0..10 {
            app.memory(unsafe { ram_data.as_ptr().add(i) as u32 })?;
        }

        app.memory_region(ram_data.as_ptr() as u32, ram_data.len() as u32 * 4)?;

        // Test PI DMA

        app.comment("Test PI DMA")?;

        let ram_data = io::Buffer::<u8>::with_alignment(0x40, n64_specs::pi::DMA_RAM_ALIGNMENT);

        io::pi_dma(
            &io::PiDma {
                direction: io::PiDmaDirection::PiToRam,
                ram_address: u24::from_u32(io::physical(ram_data.as_ptr() as u32)),
                pi_address: 0x1000_0000,
                length: u24::from_u8(0x40 - 1),
            },
            true,
        );

        app.memory_region(ram_data.as_ptr() as u32, ram_data.len() as u32)?;

        // Test RSP DMA

        // TODO

        Ok(())
    }
}
