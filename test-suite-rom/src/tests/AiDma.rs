// TODO latching
// TODO queue while disabled

use alloc::format;
use itertools::iproduct;
use n64_specs::ai;

use crate::{
    app::App,
    io, register_test,
    test::{Test, TestError},
};

register_test!(AiDma);

// Records AI DMA transfers.
//
// We can't really record the DMA output but this records the effect of a transfer on the AI registers.
//
// Findings:
// - TODO 3 mystery bits set?

#[derive(Debug)]
pub struct Dma {
    enabled: bool,
    ram_address: u32,
    length: u32,
}

impl Test for AiDma {
    type Params = Dma;

    fn cases() -> impl Iterator<Item = Self::Params> {
        let enabled = [true, false]; // TODO not needed?

        let ram_addresses = (0..10).chain([
            0x0000_0200,
            0x0000_C020,
            0x001FF_FFFE,
            0x001FF_FFFF,
            0x00200_0000,
            0x1234_5678,
            0xFE00_0000,
            0xFE00_0200,
        ]);

        let lengths = (0..10).chain([
            0x0000_0300,
            0x0003_FFFE,
            0x0003_FFFF,
            0x0004_0000,
            0xFFFF_FFFF,
            0xFFFC_0000,
            0xFFFC_0300,
        ]);

        iproduct!(enabled, ram_addresses, lengths).map(|(enabled, ram_offset, length)| Dma {
            enabled,
            ram_address: ram_offset,
            length,
        })
    }

    fn run(params: &Self::Params, app: &mut App) -> Result<(), TestError> {
        // TODO pick fast timings
        // io::write_uncached(ai::DacRate::ADDRESS, 0x5F0);
        // io::write_uncached(ai::BitRate::ADDRESS, 0xF);

        io::write_uncached(
            ai::Control::ADDRESS,
            ai::Control::default()
                .with_dma_enabled(params.enabled)
                .raw_value(),
        );

        io::write_uncached(ai::DmaRamAddress::ADDRESS, params.ram_address);

        io::write_uncached(ai::DmaLength::ADDRESS, params.length);

        io::wait_until(|| {
            !ai::Status::new_with_raw_value(io::read_uncached(ai::Status::ADDRESS)).dma_busy()
        });

        // Read the registers

        // TODO mask wc?
        app.memory_region(
            &format!(
                "DMA transfer from RAM @ {:0X} to AI of {:0X} bytes (enabled: {})",
                params.ram_address, params.length, params.enabled
            ),
            io::uncached_addr(ai::START),
            0x20,
        )
    }
}

register_test!(AiDmaQueue);

impl Test for AiDmaQueue {
    type Params = u8;

    fn cases() -> impl Iterator<Item = Self::Params> {
        [2, 3, 4, 5, 10, 100].into_iter()
    }

    fn run(dma_count: &Self::Params, app: &mut App) -> Result<(), TestError> {
        // Schedule DMA transfers

        for _ in 0..*dma_count {
            io::write_uncached(ai::DacRate::ADDRESS, 0x5F0); // TODO ?
            io::write_uncached(ai::BitRate::ADDRESS, 0xF); // TODO ?

            io::write_uncached(
                ai::Control::ADDRESS,
                ai::Control::default().with_dma_enabled(true).raw_value(),
            );

            io::write_uncached(ai::DmaRamAddress::ADDRESS, 0);

            io::write_uncached(ai::DmaLength::ADDRESS, 0x3_FFF8);
        }

        // Count how many DMA transfers really run

        let mut actual_dma_count = 0;
        let mut last_dma_length = 0;

        loop {
            let current_dma_length = io::read_uncached(ai::DmaLength::ADDRESS);

            // If the DMA length has increased, a new transfer started
            // (needs each transfer to take long enough for this sampling to work)

            if current_dma_length > last_dma_length {
                actual_dma_count += 1;
            }

            last_dma_length = current_dma_length;

            if !ai::Status::new_with_raw_value(io::read_uncached(ai::Status::ADDRESS)).dma_busy() {
                break;
            }
        }

        app.value(
            &format!("Actual DMA count after scheduling {} transfers", dma_count),
            actual_dma_count,
        )
    }
}

// register_test!(AiDmaDisabled);

// #[derive(Debug)]
// pub enum Event {
//     Enable,
//     Disable,
//     Dma,
//     Check,
// }

// impl Test for AiDmaDisabled {
//     type Params = Vec<Event>;

//     fn cases() -> impl Iterator<Item = Self::Params> {
//         [
//             alloc::vec![Event::Disable, Event::Dma],
//             // // Disabled DMAs
//             // alloc::vec![Event::Disable, Event::Dma, Event::Check],
//             // alloc::vec![Event::Disable, Event::Dma, Event::Dma, Event::Check],
//             // alloc::vec![
//             //     Event::Disable,
//             //     Event::Dma,
//             //     Event::Dma,
//             //     Event::Dma,
//             //     Event::Check
//             // ],
//             // // Disabled DMAs enabled later
//             // alloc::vec![Event::Disable, Event::Dma, Event::Enable, Event::Check],
//             // alloc::vec![Event::Disable, Event::Dma, Event::Dma, Event::Enable, Event::Check],
//             // alloc::vec![
//             //     Event::Disable,
//             //     Event::Dma,
//             //     Event::Dma,
//             //     Event::Dma,
//             //     Event::Enable
//             // ],
//             // alloc::vec![
//             //     Event::Disable,
//             //     Event::Dma,
//             //     Event::Dma,
//             //     Event::Dma,
//             //     Event::Enable
//             // ],
//             // alloc::vec![Event::Disable, Event::Dma],
//         ]
//         .into_iter()
//     }

//     fn run(_params: &Self::Params, app: &mut App) -> Result<(), TestError> {
//         io::write_uncached(ai::DacRate::ADDRESS, 0x5F0); // TODO ?
//         io::write_uncached(ai::BitRate::ADDRESS, 0xF); // TODO ?
//         io::write_uncached(ai::DmaRamAddress::ADDRESS, 0);

//         let dma = |enabled: bool| {
//             io::write_uncached(
//                 ai::Control::ADDRESS,
//                 ai::Control::default().with_dma_enabled(enabled).raw_value(),
//             );

//             io::write_uncached(ai::DmaLength::ADDRESS, 0x999);

//             // If the length changes, the transfer started

//             let mut started = false;

//             let length = io::read_uncached(ai::DmaLength::ADDRESS);

//             for _ in 0..1000 {
//                 started |= length != io::read_uncached(ai::DmaLength::ADDRESS);
//             }

//             io::wait_until(|| {
//                 !ai::Status::new_with_raw_value(io::read_uncached(ai::Status::ADDRESS)).dma_busy()
//             });

//             started
//         };

//         let enable = |enabled: bool| {
//             io::write_uncached(
//                 ai::Control::ADDRESS,
//                 ai::Control::default().with_dma_enabled(enabled).raw_value(),
//             );
//         };

//         let dma = || {
//             io::write_uncached(ai::DmaLength::ADDRESS, 0x999);
//         };

//         let dma_started = || {
//             // If the length changes, the transfer started

//             let mut started = false;

//             let length = io::read_uncached(ai::DmaLength::ADDRESS);

//             for _ in 0..1000 {
//                 started |= length != io::read_uncached(ai::DmaLength::ADDRESS);
//             }

//             started
//         };

//         let wait = || {
//             io::wait_until(|| {
//                 !ai::Status::new_with_raw_value(io::read_uncached(ai::Status::ADDRESS)).dma_busy()
//             });
//         };

//         app.comment("DMA transfer while disabled")?;
//         enable(false);
//         dma();
//         app.value(dma_started() as u32)?;
//         panic!("{:0X}", io::read_uncached(ai::Status::ADDRESS));
//         //wait();

//         Ok(())
//     }
// }
