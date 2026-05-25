//! This test records the readback behavior of the RSP's DMA registers.
//!
//! No surprises:
//! - Values written to the address registers are latched until the DMA starts
//! - Writing to the address registers while the DMA is in progress does not affect the ongoing transfer or the final register values once it completes

use arbitrary_int::u12;
use n64_specs::rsp;

use crate::{
    app::App,
    io, no_params,
    test::{Test, TestError},
};

// TODO params both dirs

pub struct RspDmaLatching;

impl Test for RspDmaLatching {
    no_params!();

    fn run(_params: &Self::Params, app: &mut App) -> Result<(), TestError> {
        // Execute a first DMA transfer to get a predictable starting state

        io::rsp_dma(&io::RspDma {
            direction: io::RspDmaDirection::RamToRsp,
            source_address: 0,
            destination_address: 0,
            rows: 0,
            length: u12::new(0x100),
            skip: u12::new(0),
        });

        io::wait_until(|| io::read_uncached(rsp::DmaBusy::ADDRESS) == 0);

        app.comment("State after initial DMA")?;
        app.value(io::read_uncached(rsp::DmaBusy::ADDRESS))?;
        app.value(io::read_uncached(rsp::DmaRspAddress::ADDRESS))?;
        app.value(io::read_uncached(rsp::DmaRamAddress::ADDRESS))?;
        app.value(io::read_uncached(rsp::DmaReadLength::ADDRESS))?;
        //app.push_value(io::read_uncached(rsp::DmaWriteLength::ADDRESS))?;

        // Setup another DMA transfer without starting it

        io::write_uncached(rsp::DmaRspAddress::ADDRESS, 0);
        app.comment("Set RSP address")?;
        app.value(io::read_uncached(rsp::DmaRspAddress::ADDRESS))?;
        app.value(io::read_uncached(rsp::DmaRamAddress::ADDRESS))?;

        io::write_uncached(rsp::DmaRamAddress::ADDRESS, 0);
        app.comment("Set RAM address")?;
        app.value(io::read_uncached(rsp::DmaRspAddress::ADDRESS))?;
        app.value(io::read_uncached(rsp::DmaRamAddress::ADDRESS))?;

        // Start the transfer

        app.comment("Start DMA")?;
        io::write_uncached(
            rsp::DmaReadLength::ADDRESS,
            rsp::DmaReadLength::default()
                .with_rows(0)
                .with_length(u12::new(0x800))
                .with_skip(u12::new(0))
                .raw_value(),
        );

        // Write random values to the address registers while the transfer is in progress

        io::write_uncached(rsp::DmaRspAddress::ADDRESS, u32::MAX);
        io::write_uncached(rsp::DmaRamAddress::ADDRESS, u32::MAX);

        // Make sure the writes occurred during the DMA

        assert_eq!(io::read_uncached(rsp::DmaBusy::ADDRESS), 1);

        io::wait_until(|| io::read_uncached(rsp::DmaBusy::ADDRESS) == 0);

        app.comment("Dma completed")?;
        app.value(io::read_uncached(rsp::DmaRspAddress::ADDRESS))?;
        app.value(io::read_uncached(rsp::DmaRamAddress::ADDRESS))?;
        app.value(io::read_uncached(rsp::DmaReadLength::ADDRESS))
        //app.push_value(io::read_uncached(rsp::DmaWriteLength::ADDRESS))?;
    }
}
