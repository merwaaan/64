//! Records DMA transfers from RAM to RSP memory.
//!
//! Findings:
//! - TODO
//!
//! No surprises:
//! - DMA wraps around the target bank without leaking into the other one

use alloc::format;
use arbitrary_int::u12;
use itertools::iproduct;
use n64_specs::rsp;

use crate::{
    app::App,
    io, register_test,
    test::{Test, TestError},
};

register_test!(RspDmaFromRam);

#[derive(Debug)]
pub struct Dma {
    ram_offset: u32,
    rsp_offset: u32,
    rows: u8,
    length: u32,
    skip: u16,
}

impl Test for RspDmaFromRam {
    type Params = Dma;

    fn cases() -> impl Iterator<Item = Self::Params> {
        // Lengths

        let with_length = (0..18).chain([128, 0x400, 0xFFE, 0xFFF]).map(|length| Dma {
            ram_offset: 0,
            rsp_offset: 0,
            rows: 0,
            length,
            skip: 0,
        });

        // RAM sources

        let with_ram_offset = (1..18).map(|ram_offset| Dma {
            ram_offset,
            rsp_offset: 0,
            rows: 0,
            length: 0x100,
            skip: 0,
        });

        // RSP destinations

        let with_rsp_offset = {
            let rsp_offsets = (1..18).chain([128, 0x400, 0xFFE, 0xFFF]);
            let rsp_bank_offsets = [0, rsp::MEMORY_BANK_SIZE]; // DMEM/IMEM

            iproduct!(rsp_offsets, rsp_bank_offsets).map(|(rsp_offset, bank_offset)| Dma {
                ram_offset: 0,
                rsp_offset: bank_offset + rsp_offset,
                rows: 0,
                length: 0x300,
                skip: 0,
            })
        };

        // Rows

        let with_rows = {
            let rows = (1..18).chain([157, 0xFF]);
            let lengths = [8, 0x200, 0xFFF];

            iproduct!(rows, lengths).map(|(rows, length)| Dma {
                ram_offset: 0,
                rsp_offset: 0x800,
                rows,
                length,
                skip: 0,
            })
        };

        // Skips

        let with_skips = {
            let skips = (0..18).chain([0x82, 0x101, 0xFFF]);
            let rows = [0, 1, 2, 15, 0xFF];

            iproduct!(skips, rows).map(|(skip, rows)| Dma {
                ram_offset: 0,
                rsp_offset: 0x500,
                rows,
                length: 0x20,
                skip,
            })
        };

        // Weird combinations

        let combos = [
            Dma {
                ram_offset: 3,
                rsp_offset: 5,
                rows: 3,
                length: 21,
                skip: 3,
            },
            Dma {
                ram_offset: 3,
                rsp_offset: 0x1FFF,
                rows: 0x7F,
                length: 2,
                skip: 0x35B,
            },
            Dma {
                ram_offset: 0,
                rsp_offset: 0x1FFF,
                rows: 0xFF,
                length: 0xFFF,
                skip: 0xFFF,
            },
        ];

        with_length
            .chain(with_rsp_offset)
            .chain(with_ram_offset)
            .chain(with_rows)
            .chain(with_skips)
            .chain(combos)

        // TODO In both directions?
    }

    fn run(dma: &Dma, app: &mut App) -> Result<(), TestError> {
        app.comment(&format!(
            "DMA transfer from RAM @ {:0X} to RSP @ {:0X}, {:0X} bytes x {:0X} rows, skip {:0X}",
            dma.ram_offset, dma.rsp_offset, dma.length, dma.rows, dma.skip
        ))?;

        // Clear the RSP memory

        for i in (0..rsp::DMEM_SIZE + rsp::IMEM_SIZE).step_by(4) {
            io::write_uncached(rsp::MEMORY_START + i, 0x0000_0000);
        }

        // Prepare some data in RAM
        // (allocate just enough space to speed up smaller tests)

        let total =
            dma.ram_offset + (dma.rows as u32 + 1) * ((dma.length | 7) + 1 + dma.skip as u32);

        let mut ram_data = io::Buffer::<u8>::with_alignment(total as usize, 8);

        for i in 0..ram_data.capacity() {
            ram_data.push(i as u8);
        }

        // DMA

        io::write_uncached(rsp::DmaRspAddress::ADDRESS, dma.rsp_offset);

        io::write_uncached(
            rsp::DmaRamAddress::ADDRESS,
            io::physical_addr(ram_data.as_ptr() as u32 + dma.ram_offset),
        );

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

        app.memory_region(
            io::uncached_addr(rsp::MEMORY_START),
            rsp::DMEM_SIZE + rsp::IMEM_SIZE,
        )
    }
}
