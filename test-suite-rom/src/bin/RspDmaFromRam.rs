//! This tests records DMA transfers from RAM to RSP memory.
//!
//! Findings:
//! -
//!
//! No surprises:
//! - DMA wraps around the target bank without leaking into the other one TODO check

#![no_std]
#![no_main]

// TODO does this wraps around RAM?

#[derive(Debug)]
struct Dma {
    rsp_destination: u32,
    rows: u8,
    length: u32,
    skip: u16,
}

test_suite_rom::run_test!(RspDmaFromRam);

const RAM_DATA_SIZE: usize = 0x4000;

impl Test for RspDmaFromRam {
    type Params = Dma;

    fn cases() -> Vec<Self::Params> {
        let mut cases = Vec::new();

        // Various destinations and lengths

        for bank_offset in [0, specs::rsp::MEMORY_BANK_SIZE] {
            for bank_internal_offset in [0, 0xD00, 0xFFF] {
                for length in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 16, 17, 128, 0x400, 0xFFF] {
                    cases.push(Dma {
                        rsp_destination: bank_offset + bank_internal_offset,
                        rows: 0,
                        length,
                        skip: 0,
                    });
                }
            }
        }

        // Various layouts

        // TODO
        // for row in [1, 2, 10, 0xFF] {
        //     cases.push(Dma {
        //         rsp_destination: 0,
        //         rows: 1,
        //         length,
        //         skip: 0,
        //     });
        // }

        cases
    }

    fn case_name(params: &Self::Params) -> String {
        format!(
            "DMA transfer to {:08X}, {} bytes x {} rows, skip {}",
            params.rsp_destination, params.length, params.rows, params.skip
        )
    }

    fn run(dma: &Dma, app: &mut App) -> Result<()> {
        // Clear the RSP memory

        let rsp_mem = io::uncached_ptr(specs::rsp::MEMORY_START);

        unsafe {
            for i in 0..0x800 {
                rsp_mem.add(i).write_volatile(0x0000_0000);
            }
        };

        // Prepare some data in RAM
        // TODO alignment a problem? #[repr(align(8))]??
        // TODO helper to allocate such blocks?

        let mut ram_data = alloc::vec![10u8; RAM_DATA_SIZE];

        let cached_ptr = ram_data.as_mut_ptr();
        let uncached_ptr = (cached_ptr as usize | 0xA000_0000) as *mut u8;

        for i in 0..RAM_DATA_SIZE {
            unsafe {
                uncached_ptr.add(i).write_volatile(i as u8);
            }
        }

        // DMA

        app.value(io::read_uncached(specs::rsp::DmaBusy::ADDRESS))?;

        io::write_uncached(specs::rsp::DmaRspAddress::ADDRESS, dma.rsp_destination);

        io::write_uncached(specs::rsp::DmaRamAddress::ADDRESS, ram_data.as_ptr() as u32);

        io::write_uncached(
            specs::rsp::DmaReadLength::ADDRESS,
            specs::rsp::DmaReadLength::default()
                .with_rows(dma.rows)
                .with_length(u12::from_u32(dma.length))
                .with_skip(u12::new(dma.skip))
                .raw_value(),
        );

        for _ in 0..3 {
            app.value(io::read_uncached(specs::rsp::DmaBusy::ADDRESS))?;
        }

        // TODO wait till it's over

        for _ in 0..1000 {
            core::hint::black_box(()); // Forces the compiler to treat this as a meaningful step
        }

        // Record the whole RSP memory

        app.memory_region(specs::rsp::MEMORY_START, 0x2000)
    }
}
