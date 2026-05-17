//! Dummy test to validate the recording mechanism and various helpers.

#![no_std]
#![no_main]

test_suite_rom::run_test!(Dummy);

impl Test for Dummy {
    type Params = bool;

    fn cases() -> Vec<Self::Params> {
        Vec::from([true, false])
    }

    fn case_name(params: &Self::Params) -> String {
        format!("With {}", params)
    }

    fn run(params: &Self::Params, app: &mut App) -> Result<()> {
        app.comment("A helpful comment")?;

        app.value(if *params { u32::MAX } else { 0 })?;

        let some_ram_data = (0..1000).map(|i| i as u32).collect::<Vec<_>>();

        for i in 0..10 {
            app.memory(unsafe { some_ram_data.as_ptr().add(i) as u32 })?;
        }

        app.memory_region(
            some_ram_data.as_ptr() as u32,
            some_ram_data.len() as u32 * 4,
        )?;

        // Test PI DMA

        app.comment("Test PI DMA")?;

        let ram_data = alloc::vec![0u8; 0x40];
        let ram_data_uncached = io::uncached_ptr(ram_data.as_ptr() as u32);

        io::pi_dma(&io::PiDma {
            direction: io::PiDmaDirection::PiToRam,
            ram_address: u24::from_u32(io::physical(ram_data.as_ptr() as u32)),
            pi_address: 0x1000_0000,
            length: u24::from_u8(0x40 - 1),
        });

        io::wait_until(|| io::read_uncached(n64_specs::pi::Status::ADDRESS) & 0x1 == 0);

        app.memory_region(ram_data_uncached as u32, ram_data.len() as u32)?;

        // Test RSP DMA

        // TODO

        Ok(())
    }
}
