use alloc::format;
use anyhow::{Result, anyhow, bail};
use test_suite_common::{Message, Step};

use crate::{display::*, io, isviewer, sc64::Sc64, test::*, tests};

#[cfg(feature = "replay")]
use crate::comparator::Comparator;

pub struct App {
    /// Graphic display.
    pub display: Display,

    /// SummerCart64 interface, if we're running on one of these.
    sc64: Option<Sc64>,

    #[cfg(feature = "replay")]
    /// Comparator for replay-mode ROMs.
    comparator: Comparator,
}

impl App {
    pub fn new() -> Result<Self> {
        let sc64 = Sc64::try_new()?;

        Ok(Self {
            display: Display::default(),
            sc64,
            #[cfg(feature = "replay")]
            comparator: Comparator::default(),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        // Display some info

        let verb = if cfg!(feature = "record") {
            "Recording"
        } else {
            "Replaying"
        };

        self.print(
            &format!(
                "{} {} tests ({} cases)\n",
                verb,
                tests::test_count(),
                tests::test_case_count()
            ),
            None,
        )?;

        if self.sc64.is_none() && cfg!(feature = "record") {
            self.print(
                "Not running on a SummerCart64, recording disabled\n",
                Some(TextStyle::with_color(WARNING)),
            )?;
        }

        // Wait for the server to be ready to receive data

        if let Some(sc64) = &mut self.sc64 {
            sc64.wait_for_server_ready_signal();
        }

        // Notify the server that the program started

        self.send(Message::ProgramStarted, false)?;

        // Run the test plan

        let failed_tests = tests::run_tests(self)?;

        let success = failed_tests.is_empty();

        // Done

        #[cfg(feature = "record")]
        {
            assert!(success, "record-mode ROM did not succeed");

            self.print("\nDone!\n", Some(TextStyle::with_color(SUCCESS)))?;
        }

        #[cfg(feature = "replay")]
        {
            if success {
                self.print("\nSuccess!\n", Some(TextStyle::with_color(SUCCESS)))?;
            } else {
                self.print(
                    &format!(
                        "\n{}/{} tests failed: {}\n",
                        failed_tests.len(),
                        tests::test_count(),
                        failed_tests.join(", ")
                    ),
                    Some(TextStyle::with_color(ERROR)),
                )?;
            }
        }

        // Notify the server that the test completed
        // (and flush any remaining buffered messages)

        self.send(Message::ProgramCompleted { success }, true)?;

        Ok(())
    }

    /// Prints text to the display and the IS-Viewer.
    pub fn print(&mut self, text: &str, style: Option<TextStyle>) -> Result<()> {
        isviewer::write(text);

        self.display.print(text, style)
    }

    /// Sends a message to the server.
    pub fn send(&mut self, message: Message, flush: bool) -> Result<()> {
        if let Some(sc64) = &mut self.sc64 {
            sc64.send(message, flush)
        } else {
            Ok(())
        }
    }

    // Helpers to emit steps

    pub fn bool(&mut self, description: &str, value: bool) -> Result<(), TestError> {
        self.process_step(Step::Bool(value), description)
    }

    pub fn value(&mut self, description: &str, value: u32) -> Result<(), TestError> {
        self.process_step(Step::Value(value), description)
    }

    pub fn value64(&mut self, description: &str, value: u64) -> Result<(), TestError> {
        self.process_step(Step::Value64(value), description)
    }

    pub fn memory(&mut self, description: &str, address: u32) -> Result<(), TestError> {
        if address & 3 != 0 {
            return Err(anyhow!(
                "Memory address ({:08X}) must be aligned to 4 bytes",
                address
            )
            .into());
        }

        let value = unsafe { (address as *const u32).read_volatile() };

        self.process_step(Step::Value(value), description)
            // In case of mismatch, add info about the address to the description
            .map_err(|e| match e {
                TestError::Mismatch(mismatch) => TestError::Mismatch(Mismatch {
                    description: format!("{} (address = {:08X})", description, address),
                    ..mismatch
                }),
                e => e,
            })
    }

    pub fn memory_region(
        &mut self,
        description: &str,
        address: u32,
        byte_length: u32,
    ) -> Result<(), TestError> {
        // NOTE: we record memory regions as separate 32-bit values to make it easier to stream
        // and prevent out-of-memory situations when serializing/deserializing large regions as a whole

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

            self.process_step(Step::Value(value), description)
                // In case of mismatch, add info about the region to the description
                .map_err(|e| match e {
                    TestError::Mismatch(mismatch) => TestError::Mismatch(Mismatch {
                        description: format!(
                            "{} (address = {:08X}, offset = {:0X})",
                            description,
                            address + (i as u32) * 4,
                            i as u32 * 4
                        ),
                        ..mismatch
                    }),
                    e => e,
                })?;
        }

        Ok(())
    }

    /// Waits for a reboot.
    ///
    /// If running on a SummerCart64, this can be triggered via the `upload rom.z64 --reboot` command.
    /// Otherwise, we'll have to reboot manually.
    pub fn wait_for_reboot(&self) -> ! {
        if let Some(sc64) = &self.sc64 {
            // Wait for the signal

            sc64.wait_for_reboot_signal();

            // To get a fresh start, we'll run IPL3 again.
            //
            // On the initial boot, the hardware-embedded IPL2 copied the start of the ROM to DMEM and execution started from there.
            // The contents of DMEM has since then been cleared (IPL3 cleaned after itself after its first invocation), so we copy it back.

            for i in (0..n64_specs::rsp::DMEM_SIZE).step_by(4) {
                io::wait_for_pi();
                let word: u32 = io::read_uncached(n64_specs::cart::START + i);
                io::write_uncached(n64_specs::rsp::DMEM_START + i, word);
            }

            // Jump to IPL3 (skip the 0x40 bytes of ROM header)

            let reboot_address = io::uncached_addr(n64_specs::rsp::DMEM_START + 0x40);

            let reboot: extern "C" fn() -> ! = unsafe { core::mem::transmute(reboot_address) };

            reboot()
        } else {
            loop {
                core::hint::spin_loop();
            }
        }
    }

    /// Runs a test.
    ///
    /// - Record mode: records the steps and sends them to the server.
    /// - Replay mode: compares the steps against the embedded recorded steps.
    pub(crate) fn run_test<T: Test>(&mut self) -> Result<bool> {
        self.print(
            &format!("{} ({} cases)", T::name(), T::cases().count()),
            None,
        )?;

        let mut success = true;

        // Run each test case

        let result = self.process_step(Step::StartTest(T::name().into()), "start test");

        success &= self.check_step::<T>(None, result)?;
        // TODO issue there? cases still running even if test failed?

        for (case_index, case) in T::cases().enumerate() {
            let result = self
                .process_step(Step::StartTestCase(case_index as u32), "start test case")
                .and_then(|_| T::run(&case, self))
                .and_then(|_| self.process_step(Step::EndTestCase, "end test case"));

            success &= self.check_step::<T>(Some(case_index), result)?;
        }

        let result = self.process_step(Step::EndTest, "end test");

        success &= self.check_step::<T>(None, result)?;

        Ok(success)
    }

    /// Helper to handle step mismatches at different levels.
    fn check_step<T: Test>(
        &mut self,
        case_index: Option<usize>,
        result: Result<(), TestError>,
    ) -> Result<bool> {
        match result {
            Ok(()) => Ok(true),

            #[cfg(feature = "record")]
            Err(TestError::Mismatch(_)) => {
                unreachable!(
                    "mismatches should not happen in record mode{}",
                    case_index
                        .map(|i| format!(" (test case {})", i))
                        .unwrap_or("".into())
                );
            }

            #[cfg(feature = "replay")]
            Err(TestError::Mismatch(mismatch)) => {
                let at = match case_index {
                    Some(case_index) => {
                        format!("case {} / step {}", case_index, mismatch.step_index)
                    }
                    None => "test".into(),
                };

                let message = format!(
                    "Mismatch at {}: {}\n  - expected {}\n  -      got {}",
                    at,
                    mismatch.description,
                    mismatch
                        .expected_step
                        .map(|s| format!("{}", s))
                        .unwrap_or("nothing".into()),
                    mismatch.runtime_step,
                );

                self.print(&message, Some(TextStyle::with_color(ERROR)))?;
                panic!("end"); // TODO temp
                // Skip to the next test/test case

                use anyhow::Context;

                match mismatch.runtime_step {
                    Step::StartTest(_) | Step::EndTest => {
                        self.comparator.skip_test().context("failed to skip test")?
                    }

                    _ => self
                        .comparator
                        .skip_case()
                        .context("failed to skip test case")?,
                }

                Ok(false)
            }

            Err(TestError::Other(e)) => bail!("failed to run test {}, {}", T::name(), e),
        }
    }

    /// "Processes" a step.
    ///
    /// - Record mode: send it to the server.
    /// - Replay mode: compare it against the embedded recorded steps.
    fn process_step(&mut self, step: Step, description: &str) -> Result<(), TestError> {
        #[cfg(feature = "record")]
        {
            if self.sc64.is_some() {
                self.send(Message::TestStep(step), false)?;
            } else {
                // If not running on a SummerCart64, logs the steps for debugging
                isviewer::write(&format!("{}: {:0X?}\n", description, step));
            }
        }

        #[cfg(feature = "replay")]
        {
            // TODO temp
            //isviewer::write(&format!("{}: {:0X?}\n", description, step));

            let comparison = self.comparator.compare(&step)?;

            // Check the outcome and convert the comparison result into an error to interrupt the test

            use crate::comparator::Comparison;
            use alloc::borrow::ToOwned;

            match comparison {
                Comparison::Same => {}

                Comparison::Different {
                    expected_step,
                    step_index,
                } => {
                    return Err(TestError::Mismatch(Mismatch {
                        runtime_step: step,
                        expected_step: Some(expected_step.clone()),
                        step_index,
                        description: description.to_owned(),
                    }));
                }

                Comparison::TooManySteps { step_index } => {
                    return Err(TestError::Mismatch(Mismatch {
                        runtime_step: step,
                        expected_step: None,
                        step_index,
                        description: description.to_owned(),
                    }));
                }
            }
        }

        Ok(())
    }
}
