use alloc::string::String;
use anyhow::Result;
use test_suite_common::{Message, Step};

use crate::{display::Display, sc64::Sc64};

pub struct App {
    pub display: Display,
    sc64: Sc64,
}

impl Default for App {
    fn default() -> Self {
        Self {
            display: Display::default(),
            sc64: Sc64::default(),
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

    // Helpers to push test steps

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
        assert!(
            address & 3 == 0,
            "Memory address ({:08X}) must be aligned to 4 bytes",
            address
        );

        let value = unsafe { (address as *const u32).read_volatile() };

        self.process_step(Step::Memory { address, value })
    }

    pub fn memory_region(&mut self, address: u32, byte_length: u32) -> Result<()> {
        assert!(
            address & 3 == 0,
            "Memory address ({:08X}) must be aligned to 4 bytes",
            address
        );

        let address_ptr = address as *const u32;

        let word_length = ((byte_length + 3) >> 2) as usize;

        let mut values = alloc::vec![0u32; word_length];
        let values_ptr = values.as_mut_ptr() as *mut u32;

        unsafe {
            for i in 0..word_length {
                let value = address_ptr.add(i).read_volatile();

                values_ptr.add(i).write_volatile(value);
            }
        };

        self.process_step(Step::MemoryRegion { address, values })
    }

    /// "Processes" a step.
    ///
    /// - Record mode: send it to the server.
    /// - Compare mode: compare against the embedded recorded steps.
    fn process_step(&mut self, step: Step) -> Result<()> {
        if cfg!(feature = "record") {
            self.send(Message::TestStep(step))?;
        } else {
            // TODO
        }

        Ok(())
    }
}
