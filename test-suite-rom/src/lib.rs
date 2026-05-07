#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

#[cfg(not(any(feature = "record", feature = "compare")))]
compile_error!("must enable either feature \"record\" or \"compare\"");

#[cfg(all(feature = "record", feature = "compare"))]
compile_error!("features \"record\" and \"compare\" are mutually exclusive");

extern crate alloc;

pub mod allocator;
pub mod framebuffer;
pub mod sc64;

/// Macro that defines a test implementing the `Test` trait and wires it to the entrypoint.
#[macro_export]
macro_rules! define_test {
    ($test:ident { $($body:tt)* }) => {
        extern crate alloc;

        use alloc::{format, string::*, vec::*, vec};
        use anyhow::{anyhow, Result};
        use n64_specs as specs;
        use test_suite_common::*;
        use test_suite_rom::*;

        static mut FRAMEBUFFER: *mut $crate::framebuffer::Framebuffer = core::ptr::null_mut();

        fn framebuffer() -> &'static mut $crate::framebuffer::Framebuffer {
            unsafe { &mut *FRAMEBUFFER }
        }

        #[panic_handler]
        fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
            framebuffer().print(&alloc::format!("{}", info), Some($crate::framebuffer::ERROR)).ok();
            framebuffer().frame(false).ok();

            $crate::sc64::Sc64::send(Message::Panic).ok();

            $crate::sc64::Sc64::wait_for_reboot();
        }

        struct $test;

        impl Test for $test {
            $($body)*
        }

        #[unsafe(no_mangle)]
        extern "C" fn _entrypoint() -> ! {
            // Setup the global allocator

            $crate::allocator::configure();

            // Setup the framebuffer

            let mut fb = alloc::boxed::Box::new($crate::framebuffer::Framebuffer::new());

            unsafe { FRAMEBUFFER = &raw mut *fb; }

            // Start the main loop

            match main_loop::<$test>() {
                Ok(()) => $crate::sc64::Sc64::wait_for_reboot(),
                Err(e) => panic!("{e:#}"),
            }
        }

        fn main_loop<T: Test>() -> Result<()> {
            let (mode, verb) = if cfg!(feature = "record") {
                ("record", "Recording")
            } else {
                ("compare", "Comparing")
            };

            framebuffer().print(&alloc::format!("{} (mode: {})\n", T::name(), mode), None)?;

            // Run the test

            framebuffer().print(&alloc::format!("{} {} test case{}...", verb, T::cases().len(), if T::cases().len() == 1 { "" } else { "s" }), None)?;

            let result = T::run();

            // Record or compare the results

            #[cfg(feature = "record")]
            record(result)?;

            #[cfg(feature = "compare")]
            compare(result)?;

            Ok(())
        }

        /// Record mode: sends the test results to the server.
        #[cfg(feature = "record")]
        fn record(result: TestResult) -> Result<()> {
            framebuffer().print("Sending results over USB...", None)?;

            $crate::sc64::Sc64::configure()?;
            $crate::sc64::Sc64::send(Message::TestResult(result));

            framebuffer().print("\nDone!\n", Some($crate::framebuffer::SUCCESS))?;
            framebuffer().frame(true)
        }

        /// Compare mode: compares the test results with the embedded results recorded on hardware.
        #[cfg(feature = "compare")]
        fn compare(result: TestResult) -> Result<()> {
            let reference_result_data = include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../_test_suite_output/",
                stringify!($test),
                ".bin"
            ));

            let reference_result: TestResult = postcard::from_bytes(reference_result_data)
                .map_err(|e| anyhow!("failed to deserialize embedded result: {e}"))?;

            let success = result == reference_result;

            if success {
                framebuffer().print("\nSuccess!\n", Some($crate::framebuffer::SUCCESS))?;
            } else {
                framebuffer().print("\nFailure!\n", Some($crate::framebuffer::WARNING))?;

                if let Some(diff) = result.first_diff(&reference_result) {
                    framebuffer().print(&diff, Some($crate::framebuffer::ERROR))?;
                }
            }

            framebuffer().frame(success)
        }
    };
}

// TODO mvoe to io helpers
pub fn reg_mut_ptr(offset: u32) -> *mut u32 {
    (n64_specs::map::Segment::KSEG1 as u32 | offset) as *mut u32
}
