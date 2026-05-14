//! This test queues multiple RSP DMA transfers to record how they buffer.
//!
//! No surprises:
//! - ?

#![no_std]
#![no_main]

// TODO params both dirs

test_suite_rom::run_test!(RspDmaBuffering);

impl Test for RspDmaBuffering {
    type Params = u32;

    fn cases() -> Vec<Self::Params> {
        Vec::from([2 /* , 3, 4, 10, 100*/])
    }

    fn case_name(params: &Self::Params) -> String {
        format!("Queue {} DMA transfers", params)
    }

    fn run(transfers: &Self::Params, app: &mut App) -> Result<()> {
        // Fill the RAM with sequences of unique values.
        // Each transfer will be assigned a different sequence so that we can identify which one copied what.

        let ram_size = (specs::rsp::MEMORY_BANK_SIZE * *transfers) as usize;

        let mut ram_data = alloc::vec![1u8; ram_size];

        // let cached_ptr = ram_data.as_mut_ptr();
        // let uncached_ptr = (cached_ptr as usize | 0xA000_0000) as *mut u8;

        // unsafe {
        //     for transfer in 0..*transfers {
        //         for i in 0..specs::rsp::MEMORY_BANK_SIZE {
        //             uncached_ptr
        //                 //.add((transfer * specs::rsp::MEMORY_BANK_SIZE + i) as usize)
        //                 .write_volatile(5 as u8);
        //         }
        //     }
        // }

        let cached_ptr = ram_data.as_mut_ptr();
        let uncached_ptr = (cached_ptr as usize | 0xA000_0000) as *mut u32;

        unsafe {
            uncached_ptr
                //.add((transfer * specs::rsp::MEMORY_BANK_SIZE + i) as usize)
                .write_volatile(5);
        }

        // for i in 0..ram_size {
        //     app.push_value(unsafe { uncached_ptr.add(i).read_volatile() as u32 })?;
        // }
        app.push_memory_region(uncached_ptr as u32, ram_size as u32)

        //app.push_memory_region(ram_data.as_ptr() as u32, ram_data.len() as u32)

        // app.push_memory_region(sources[0].as_ptr() as u32, specs::rsp::MEMORY_BANK_SIZE)

        // // Clear the last byte of the RSP memory to use it as a sentinel

        // // Queue all the DMA transfers

        // for i in 0..*transfers {
        //     io::dma_ram_to_rsp(&io::RspDma {
        //         direction: io::RspDmaDirection::RamToRsp,
        //         source_address: i * 100,
        //         destination_address: 0,
        //         rows: 0,
        //         length: u12::new(0xFFF),
        //         skip: u12::new(0),
        //     });

        //     // TODO busy/full

        //     // Make sure that the transfers are queued while the first one is still in progress

        //     assert_eq!(io::read_uncached(n64_specs::rsp::DmaBusy::ADDRESS), 1);
        // }

        // io::wait_until(|| io::read_uncached(n64_specs::rsp::DmaBusy::ADDRESS) == 0);

        // // Record the final RAM address, which indicates which transfer completed last

        // app.push_value(io::read_uncached(specs::rsp::DmaRamAddress::ADDRESS))
    }
}
