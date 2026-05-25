//! Records DMA transfers from RAM to RSP memory.
//!
//! Findings:
//! -
//!
//! No surprises:
//! - DMA wraps around the target bank without leaking into the other one TODO check

use alloc::format;
use arbitrary_int::u12;
use n64_specs::rsp;

use crate::{
    app::App,
    io, register_test,
    test::{Test, TestError},
};

// TODO does this wraps around RAM?

register_test!(RspDmaFromRam);

#[derive(Debug)]
pub struct Dma {
    ram_offset: u32,
    rsp_offset: u32,
    rows: u8,
    length: u32,
    skip: u16,
}

const RAM_DATA_SIZE: usize = 0x4000;

// TODO what if non aligned? exception? buggy?

impl Test for RspDmaFromRam {
    type Params = Dma;

    fn cases() -> impl Iterator<Item = Self::Params> {
        let ram_offset = [0, 8, 0x100];
        let rsp_bank_offsets = [0, rsp::MEMORY_BANK_SIZE];
        let rsp_offsets = [0, 0xD00, 0xFFF];
        let lengths = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 16, 17, 128, 0x400, 0xFFF];

        itertools::iproduct!(ram_offset, rsp_bank_offsets, rsp_offsets, lengths).map(
            |(ram_offset, bank_offset, destination, length)| Dma {
                ram_offset,
                rsp_offset: bank_offset + destination,
                rows: 0,
                length,
                skip: 0,
            },
        )

        // TODO rows, skip
    }

    fn run(dma: &Dma, app: &mut App) -> Result<(), TestError> {
        app.comment(&format!(
            "DMA transfer from RAM @ {:08X} to RSP @ {:08X}, {} bytes x {} rows, skip {}",
            dma.ram_offset, dma.rsp_offset, dma.length, dma.rows, dma.skip
        ))?;

        // Clear the RSP memory

        let rsp_mem = io::uncached_ptr(rsp::MEMORY_START);

        for i in (0..rsp::DMEM_SIZE + rsp::IMEM_SIZE).step_by(4) {
            io::write_uncached(rsp_mem as u32 + i, 0x0000_0000);
        }

        // Prepare some data in RAM

        let mut ram_data = io::Buffer::<u8>::with_alignment(RAM_DATA_SIZE, 8);

        for i in 0..RAM_DATA_SIZE {
            ram_data.set(i, i as u8);
        }

        // DMA

        io::write_uncached(rsp::DmaRspAddress::ADDRESS, dma.rsp_offset);

        io::write_uncached(rsp::DmaRamAddress::ADDRESS, unsafe {
            ram_data.as_ptr().add(dma.ram_offset as usize) as u32
        });

        io::write_uncached(
            rsp::DmaReadLength::ADDRESS,
            rsp::DmaReadLength::default()
                .with_rows(dma.rows)
                .with_length(u12::from_u32(dma.length))
                .with_skip(u12::new(dma.skip))
                .raw_value(),
        );

        io::wait_until(|| io::read_uncached(rsp::DmaBusy::ADDRESS) == 0);

        // Record the whole RSP memory

        app.memory_region(rsp::MEMORY_START, rsp::DMEM_SIZE + rsp::IMEM_SIZE)
    }
}
