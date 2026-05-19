use alloc::string::String;
use anyhow::{Result, anyhow, bail};
use test_suite_common::{Message, Step};

use crate::{
    display::{Display, SUCCESS},
    sc64::Sc64,
    test::{Test, TestError},
};

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
    pub fn run<T: Test>(&mut self) -> Result<()> {
        // Display the test name and mode

        const MODE: &str = if cfg!(feature = "record") {
            "record"
        } else {
            "compare"
        };

        self.display
            .print(&alloc::format!("{} (mode: {})\n", T::name(), MODE), None)?;

        // Run the test

        self.run_test::<T>()?;

        // Done!

        self.display.print("\nDone!\n", Some(SUCCESS))?;
        self.display.frame(true)
    }

    /// Sends a message to the server.
    pub fn send(&self, message: Message) -> Result<()> {
        self.sc64.send(message)
    }

    /// Indefinitely waits for the SC64 to reboot.
    pub fn wait_for_reboot(&self) -> ! {
        self.sc64.wait_for_reboot()
    }

    // Helpers to emit steps

    pub fn comment(&mut self, comment: &str) -> Result<(), TestError> {
        self.process_step(Step::Comment(String::from(comment)))
    }

    pub fn value(&mut self, value: u32) -> Result<(), TestError> {
        self.process_step(Step::Value(value))
    }

    pub fn memory(&mut self, address: u32) -> Result<(), TestError> {
        if address & 3 != 0 {
            return Err(anyhow!(
                "Memory address ({:08X}) must be aligned to 4 bytes",
                address
            )
            .into());
        }

        let value = unsafe { (address as *const u32).read_volatile() };

        self.process_step(Step::Value(value))
    }

    pub fn memory_region(&mut self, address: u32, byte_length: u32) -> Result<(), TestError> {
        if address & 3 != 0 {
            return Err(anyhow!(
                "Memory address ({:08X}) must be aligned to 4 bytes",
                address
            )
            .into());
        }

        let address_ptr = address as *const u32;

        let word_length = ((byte_length + 3) >> 2) as usize;

        for i in 0..word_length {
            let value = unsafe { address_ptr.add(i).read_volatile() };

            self.process_step(Step::Value(value))?;
        }

        Ok(())
    }

    /// Runs a test.
    ///
    /// - Record mode: records the steps and sends them to the server.
    /// - Compare mode: compares the steps against the embedded recorded steps.
    fn run_test<T: Test>(&mut self) -> Result<()> {
        #[cfg(feature = "record")]
        self.record_test::<T>()?;

        #[cfg(feature = "compare")]
        self.compare_test::<T>()?;

        Ok(())
    }

    #[cfg(feature = "record")]
    fn record_test<T: Test>(&mut self) -> Result<()> {
        // Notify the server that the test starts

        self.send(Message::TestStarted)?;

        // Run all the test cases

        for (case_index, params) in T::cases().iter().enumerate() {
            self.display
                .print(&alloc::format!("Running case #{}...", case_index), None)?;

            // Record a "case start" step to delimit cases

            self.process_step(Step::StartTestCase)?;

            // Run the test case

            match T::run(params, self) {
                Ok(()) => {}

                Err(e) => bail!(
                    "failed to run test case #{} with params {:?}, {}",
                    case_index,
                    params,
                    123, // TODOe
                ),
            }
        }

        // Notify the server that the test completed

        self.send(Message::TestCompleted)
    }

    #[cfg(feature = "compare")]
    fn compare_test<T: Test>(&mut self) -> Result<()> {
        // Run all the test cases

        for (case_index, params) in T::cases().iter().enumerate() {
            self.display
                .print(&alloc::format!("Running case #{}...", case_index), None)?;

            match T::run(&params, self) {
                Ok(()) => {
                    // Ensure that all the recorded steps have been compared

                    match self.comparator.finalize_case() {
                        Ok(()) => {}

                        Err(TestError::Mismatch(mismatch)) => {
                            self.display.print(
                                &alloc::format!("mismatch {:?}", mismatch),
                                Some(crate::display::ERROR),
                            )?;

                            self.comparator.skip_case()?;
                        }

                        Err(TestError::Other(e)) => {
                            bail!("failed to finalize test case #{}, {}", case_index, e)
                        }
                    }

                    self.display.print("ok", None)?;
                }

                Err(TestError::Mismatch(mismatch)) => {
                    self.display.print(
                        &alloc::format!("mismatch {:?}", mismatch),
                        Some(crate::display::ERROR),
                    )?;

                    // Skip the rest of the test case
                    // (only if the mismatch is not due to excess steps, in which case we're already at the start of the next case)

                    if !matches!(mismatch, crate::test::Mismatch::ExcessSteps { .. }) {
                        self.comparator.skip_case()?;
                    }
                }

                Err(TestError::Other(e)) => bail!("failed to run test case #{}, {}", case_index, e),
            }
        }

        Ok(())
    }

    /// "Processes" a step.
    ///
    /// - Record mode: send it to the server.
    /// - Compare mode: compare it against the embedded recorded steps.
    fn process_step(&mut self, step: Step) -> Result<(), TestError> {
        #[cfg(feature = "record")]
        {
            self.send(Message::TestStep(step))?;
        }

        #[cfg(feature = "compare")]
        {
            self.comparator.compare(&step)?;
        }

        Ok(())
    }
}
