use alloc::string::String;
use anyhow::{Result, bail};
use test_suite_common::{Message, Step};

use crate::{display::Display, sc64::Sc64};

#[cfg(feature = "compare")]
use crate::comparator::Comparator;

pub struct App {
    pub display: Display,
    sc64: Sc64,

    #[cfg(feature = "compare")]
    comparator: Comparator,
}

impl Default for App {
    fn default() -> Self {
        Self {
            display: Display::default(),
            sc64: Sc64::default(),

            #[cfg(feature = "compare")]
            comparator: Comparator::default(),
        }
    }
}

impl App {
    /// Sends a message to the server.
    pub fn send(&self, message: Message) -> Result<()> {
        self.sc64.send(message)
    }

    /// Indefinitely waits for the SC64 to reboot.
    pub fn wait_for_reboot(&self) -> ! {
        self.sc64.wait_for_reboot()
    }

    pub fn test_case(&mut self, name: String) -> Result<()> {
        self.process_step(Step::TestCase { name })
    }

    pub fn comment(&mut self, comment: &str) -> Result<()> {
        self.process_step(Step::Comment(String::from(comment)))
    }

    pub fn value(&mut self, value: u32) -> Result<()> {
        self.process_step(Step::Value(value))
    }

    pub fn memory(&mut self, address: u32) -> Result<()> {
        if address & 3 != 0 {
            bail!(
                "Memory address ({:08X}) must be aligned to 4 bytes",
                address
            );
        }

        let value = unsafe { (address as *const u32).read_volatile() };

        self.process_step(Step::Value(value))
    }

    pub fn memory_region(&mut self, address: u32, byte_length: u32) -> Result<()> {
        if address & 3 != 0 {
            bail!(
                "Memory address ({:08X}) must be aligned to 4 bytes",
                address
            );
        }

        let address_ptr = address as *const u32;

        let word_length = ((byte_length + 3) >> 2) as usize;

        for i in 0..word_length {
            let value = unsafe { address_ptr.add(i).read_volatile() };

            self.process_step(Step::Value(value))?;
        }

        Ok(())
    }

    /// "Processes" a step.
    ///
    /// - Record mode: send it to the server.
    /// - Compare mode: compare it against the embedded recorded steps.
    fn process_step(&mut self, step: Step) -> Result<()> {
        #[cfg(feature = "record")]
        self.record_step(step)?;

        #[cfg(feature = "compare")]
        self.compare_step(step)?;

        Ok(())

        // if cfg!(feature = "record") {
        //     self.send(Message::TestStep(step))?;
        // } else if cfg!(feature = "compare") {
        //     self.dma_streamer.compare(&step)?;

        //     // The offset in ROM of the embedded data produced by the corresponding record-mode test.
        //     //
        //     // The build process appends the data to the ROM and patches this symbol to the actual offset.
        //     #[used(linker)]
        //     #[unsafe(no_mangle)]
        //     static EMBEDDED_DATA_ROM_OFFSET: u32 = 0x0BAD_0BAD;

        //     // Get the offset from a runtime memory read to prevent the compiler from const-folding the value and breaking patching
        //     let embedded_data_rom_offset =
        //         unsafe { (&raw const EMBEDDED_DATA_ROM_OFFSET as *const u32).read_volatile() };

        //     let ram_data = alloc::vec![0u8; 0x1000];

        //     io::pi_dma(&io::PiDma {
        //         direction: io::PiDmaDirection::PiToRam,
        //         ram_address: u24::from_u32(io::physical(ram_data.as_ptr() as u32)),
        //         pi_address: 0x1000_0000 | embedded_data_rom_offset,
        //         length: u24::from_u8(0x40 - 1),
        //     });

        //     io::wait_until(|| io::read_uncached(n64_specs::pi::Status::ADDRESS) & 0x1 == 0);

        //     let ram_data_uncached = io::uncached_ptr(ram_data.as_ptr() as u32);

        //     let mut buffer: alloc::vec::Vec<u8> = alloc::vec::Vec::with_capacity(0x40 * 4);

        //     for i in 0..0x40 {
        //         // self.display.print(
        //         //     &alloc::format!("{:08X}", unsafe {
        //         //         ram_data_uncached.add(i).read_volatile()
        //         //     }),
        //         //     None,
        //         // )?;

        //         buffer.extend_from_slice(&unsafe {
        //             ram_data_uncached.add(i).read_volatile().to_be_bytes()
        //         });
        //     }

        //     // self.display
        //     //     .print(&alloc::format!("{:0X?}", &buffer[0..0x40]), None)?;

        //     let (step, rest) = postcard::take_from_bytes::<Step>(&buffer)?;
        //     self.display.print(&alloc::format!("{:0X?}", &step), None)?;

        //     let (step, rest) = postcard::take_from_bytes::<Step>(&rest)?;
        //     self.display.print(&alloc::format!("{:0X?}", &step), None)?;

        //     let (step, rest) = postcard::take_from_bytes::<Step>(&rest)?;
        //     self.display.print(&alloc::format!("{:0X?}", &step), None)?;
        // }

        // Ok(())
    }

    #[cfg(feature = "record")]
    fn record_step(&mut self, step: Step) -> Result<()> {
        self.send(Message::TestStep(step))
    }

    #[cfg(feature = "compare")]
    fn compare_step(&mut self, step: Step) -> Result<()> {
        let s = self.comparator.compare(&step)?;

        self.display
            .print(&alloc::format!("- {:?} {:?}", step, s), None)
    }
}
