use alloc::{string::String, vec::Vec};
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
    pub fn send(&self, message: Message) -> Result<()> {
        self.sc64.send(message)
    }

    pub fn wait_for_reboot(&self) -> ! {
        self.sc64.wait_for_reboot()
    }

    // Test steps
    // TODO just push_step + helpers in step?

    // TODO make a message?
    pub fn push_test_case(&mut self, name: String) -> Result<()> {
        self.send(Message::TestStep(Step::TestCase { name }))
    }

    pub fn push_comment(&mut self, comment: &str) -> Result<()> {
        self.send(Message::TestStep(Step::Comment(String::from(comment))))
    }

    pub fn push_value(&mut self, value: u32) -> Result<()> {
        self.send(Message::TestStep(Step::Value(value)))
    }

    pub fn push_pc(&mut self) -> Result<()> {
        self.send(Message::TestStep(Step::Pc(0)))
    }

    pub fn push_memory(&mut self, address: u32) -> Result<()> {
        assert!(
            address & 3 == 0,
            "Memory address ({:08X}) must be aligned to 4 bytes",
            address
        );

        let value = unsafe { (address as *const u32).read_volatile() };

        self.send(Message::TestStep(Step::Memory { address, value }))
    }

    pub fn push_memory_region(&mut self, address: u32, byte_length: u32) -> Result<()> {
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

        self.send(Message::TestStep(Step::MemoryRegion { address, values }))
    }
}
