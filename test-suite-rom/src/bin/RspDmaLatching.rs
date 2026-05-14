//! This test records the readback behavior of the RSP's DMA registers.
//!
//! Findings:
//! - Values written to the address registers are latched until the DMA starts
//! - Writing to the address registers while the DMA is in progress does not affect the ongoing transfer or the final register values once it completes

#![no_std]
#![no_main]

// TODO params both dirs

test_suite_rom::run_test!(RspDmaLatching);

impl Test for RspDmaLatching {
    no_params!();

    fn run(_params: &Self::Params, app: &mut App) -> Result<()> {
        // Execute a first DMA transfer to get a predictable starting state

        io::dma_ram_to_rsp(&io::RspDma {
            direction: io::RspDmaDirection::RamToRsp,
            source_address: 0,
            destination_address: 0,
            rows: 0,
            length: u12::new(0x100),
            skip: u12::new(0),
        });

        io::wait_until(|| io::read_uncached(n64_specs::rsp::DmaBusy::ADDRESS) == 0);

        app.push_comment("State after initial DMA")?;
        app.push_value(io::read_uncached(specs::rsp::DmaBusy::ADDRESS))?;
        app.push_value(io::read_uncached(specs::rsp::DmaRspAddress::ADDRESS))?;
        app.push_value(io::read_uncached(specs::rsp::DmaRamAddress::ADDRESS))?;
        app.push_value(io::read_uncached(specs::rsp::DmaReadLength::ADDRESS))?;
        //app.push_value(io::read_uncached(specs::rsp::DmaWriteLength::ADDRESS))?;

        // Setup another DMA transfer without starting it

        io::write_uncached(n64_specs::rsp::DmaRspAddress::ADDRESS, 0);
        app.push_comment("Set RSP address")?;
        app.push_value(io::read_uncached(specs::rsp::DmaRspAddress::ADDRESS))?;
        app.push_value(io::read_uncached(specs::rsp::DmaRamAddress::ADDRESS))?;

        io::write_uncached(n64_specs::rsp::DmaRamAddress::ADDRESS, 0);
        app.push_comment("Set RAM address")?;
        app.push_value(io::read_uncached(specs::rsp::DmaRspAddress::ADDRESS))?;
        app.push_value(io::read_uncached(specs::rsp::DmaRamAddress::ADDRESS))?;

        // Start the transfer

        app.push_comment("Start DMA")?;
        io::write_uncached(
            n64_specs::rsp::DmaReadLength::ADDRESS,
            n64_specs::rsp::DmaReadLength::default()
                .with_rows(0)
                .with_length(u12::new(0x800))
                .with_skip(u12::new(0))
                .raw_value(),
        );

        // Write random values to the address registers while the transfer is in progress

        io::write_uncached(n64_specs::rsp::DmaRspAddress::ADDRESS, u32::MAX);
        io::write_uncached(n64_specs::rsp::DmaRamAddress::ADDRESS, u32::MAX);

        // Make sure the writes occurred during the DMA

        assert_eq!(io::read_uncached(n64_specs::rsp::DmaBusy::ADDRESS), 1);

        io::wait_until(|| io::read_uncached(n64_specs::rsp::DmaBusy::ADDRESS) == 0);

        app.push_comment("Dma completed")?;
        app.push_value(io::read_uncached(specs::rsp::DmaRspAddress::ADDRESS))?;
        app.push_value(io::read_uncached(specs::rsp::DmaRamAddress::ADDRESS))?;
        app.push_value(io::read_uncached(specs::rsp::DmaReadLength::ADDRESS))
        //app.push_value(io::read_uncached(specs::rsp::DmaWriteLength::ADDRESS))?;
    }
}
