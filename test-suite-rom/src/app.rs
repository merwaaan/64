use alloc::{format, string::String};
use anyhow::{Result, anyhow, bail};
use test_suite_common::{Message, Step};

use crate::{display::*, io, isviewer, sc64::Sc64, test::*};

#[cfg(feature = "replay")]
use crate::comparator::Comparator;

pub struct App {
    /// Graphic display.
    pub display: Display,

    /// SummerCart64 interface, if we're running on one of these.
    sc64: Option<Sc64>,

    #[cfg(feature = "replay")]
    /// Comparator used in replay-mode ROMS.
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

    pub fn run<T: Test>(&mut self) -> Result<()> {
        // Display some info

        let (mode, verb) = if cfg!(feature = "record") {
            ("record", "Recording")
        } else {
            ("replay", "Replaying")
        };

        self.print(&format!("{} (mode: {})\n", T::name(), mode), None)?;

        if self.sc64.is_none() && cfg!(feature = "record") {
            self.print(
                "Not running on a SummerCart64, recording disabled\n",
                Some(TextStyle::with_color(WARNING)),
            )?;
        }

        // Run the test

        self.print(
            &format!("{} {} test cases...", verb, T::cases().count()),
            None,
        )?;

        self.run_test::<T>()
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

    pub fn start_test_case(&mut self) -> Result<(), TestError> {
        self.process_step(Step::StartTestCase)
    }

    pub fn end_test_case(&mut self) -> Result<(), TestError> {
        self.process_step(Step::EndTestCase)
    }

    pub fn comment(&mut self, comment: &str) -> Result<(), TestError> {
        self.process_step(Step::Comment(String::from(comment)))
    }

    pub fn value(&mut self, value: u32) -> Result<(), TestError> {
        self.process_step(Step::Value(value))
    }

    pub fn value64(&mut self, value: u64) -> Result<(), TestError> {
        self.process_step(Step::Value64(value))
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

            self.process_step(Step::Value(value))?;
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

            for i in (0..0x1000).step_by(4) {
                io::wait_for_pi();
                let word = io::read_uncached(n64_specs::cart::START + i);
                io::write_uncached(n64_specs::rsp::DMEM_START + i, word);
            }

            // Jump to IPL3 (skip the 0x40 bytes of ROM header)

            let reboot_address = io::uncached_ptr(n64_specs::rsp::DMEM_START + 0x40);

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
    fn run_test<T: Test>(&mut self) -> Result<()> {
        #[cfg(feature = "record")]
        self.record_test::<T>()?;

        #[cfg(feature = "replay")]
        self.replay_test::<T>()?;

        Ok(())
    }

    #[cfg(feature = "record")]
    fn record_test<T: Test>(&mut self) -> Result<()> {
        // Wait for the server to be ready to receive data

        if let Some(sc64) = &mut self.sc64 {
            sc64.wait_for_server_ready_signal();
        }

        // Notify the server that the test starts

        self.send(Message::TestStarted, false)?;

        // Run each test case

        for (case_index, params) in T::cases().enumerate() {
            let result = self
                .process_step(Step::StartTestCase)
                .and_then(|_| T::run(&params, self))
                .and_then(|_| self.process_step(Step::EndTestCase));

            match result {
                Ok(()) => {}

                Err(TestError::Other(e)) => bail!(
                    "failed to run test case {} with params {:?}, {}",
                    case_index,
                    params,
                    e
                ),

                Err(TestError::Mismatch(_)) => {
                    unreachable!("mismatches should not happen in record mode")
                }
            }

            if case_index % 100 == 0 {
                self.display
                    .progress(case_index as u32 + 1, T::cases().count() as u32)?;
            }
        }

        // Notify the server that the test completed
        // (and flush any remaining buffered messages)

        self.send(Message::TestCompleted, true)?;

        // Done!

        self.print("\nDone!\n", Some(TextStyle::with_color(SUCCESS)))
    }

    #[cfg(feature = "replay")]
    fn replay_test<T: Test>(&mut self) -> Result<()> {
        // Run each test case

        let mut successful_cases = 0;

        for (case_index, params) in T::cases().enumerate() {
            let result = self
                .process_step(Step::StartTestCase)
                .and_then(|_| T::run(&params, self))
                .and_then(|_| self.process_step(Step::EndTestCase));

            match result {
                Ok(()) => {
                    successful_cases += 1;
                }

                Err(TestError::Mismatch(mismatch)) => {
                    // Display

                    let message = format!(
                        "Mismatch at case {} / step {}:\n  - expected {}\n  -      got {:08X?}",
                        mismatch.case_index,
                        mismatch.step_index,
                        mismatch
                            .expected_step
                            .map(|s| format!("{:08X?}", s))
                            .unwrap_or("nothing".into()),
                        mismatch.runtime_step,
                    );

                    self.display
                        .print(&message, Some(TextStyle::with_color(ERROR)))?;
                }

                Err(TestError::Other(e)) => bail!("failed to run test case {}, {}", case_index, e),
            }

            // Skip to the next case

            use anyhow::Context;

            self.comparator
                .skip_case()
                .with_context(|| format!("failed to skip test case {}", case_index))?;

            if case_index % 100 == 0 {
                self.display
                    .progress(case_index as u32 + 1, T::cases().count() as u32)?;
            }
        }

        // Done!

        if successful_cases == T::cases().count() {
            self.print("\nSuccess!\n", Some(TextStyle::with_color(SUCCESS)))?;
        } else {
            self.print(
                &format!(
                    "\n{}/{} tests failed\n",
                    T::cases().count() - successful_cases,
                    T::cases().count()
                ),
                Some(TextStyle::with_color(ERROR)),
            )?;
        }

        Ok(())
    }

    /// "Processes" a step.
    ///
    /// - Record mode: send it to the server.
    /// - Replay mode: compare it against the embedded recorded steps.
    fn process_step(&mut self, step: Step) -> Result<(), TestError> {
        #[cfg(feature = "record")]
        {
            self.send(Message::TestStep(step), false)?;
        }

        #[cfg(feature = "replay")]
        {
            self.comparator.compare(&step)?;
        }

        Ok(())
    }
}
