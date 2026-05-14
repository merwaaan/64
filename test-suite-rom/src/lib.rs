#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(asm_experimental_arch)]
#![feature(associated_type_defaults)]

#[cfg(not(any(feature = "record", feature = "compare")))]
compile_error!("must enable either feature \"record\" or \"compare\"");

#[cfg(all(feature = "record", feature = "compare"))]
compile_error!("features \"record\" and \"compare\" are mutually exclusive");

pub mod allocator;
pub mod app;
pub mod display;
pub mod io;
pub mod sc64;
pub mod test;

extern crate alloc;

use crate::app::App;

// Global app instance (for accessing it from the panic handler)
static mut APP: *mut App = core::ptr::null_mut();

pub fn app() -> &'static mut App {
    unsafe { &mut *APP }
}

pub fn init_app() -> &'static mut App {
    crate::allocator::configure();

    let app_boxed = alloc::boxed::Box::new(App::default());

    unsafe {
        APP = &raw mut *alloc::boxed::Box::into_raw(app_boxed);
    }

    app()
}

#[macro_export]
macro_rules! run_test {
    {$test:ident} => {
        extern crate alloc;

        // Import useful types
        use alloc::{format, string::*, vec::Vec};
        use arbitrary_int::prelude::*;
        use anyhow::Result;
        use core::arch::asm;
        use n64_specs as specs;
        use test_suite_common::*;
        use test_suite_rom::*;
        use crate::{app::App, test::Test};

        #[panic_handler]
        fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
            $crate::app().display.print(&alloc::format!("{}", info), Some($crate::display::ERROR)).ok();
            $crate::app().display.frame(false).ok();

            $crate::app().send(Message::Panic).ok();

            $crate::app().wait_for_reboot()
        }

        struct $test;

        #[unsafe(no_mangle)]
        extern "C" fn _entrypoint() -> ! {
            // Initialize the app

            let mut app = $crate::init_app();

            // Run the test

            let (mode, verb) = if cfg!(feature = "record") {
                ("record", "Recording")
            } else {
                ("compare", "Comparing")
            };

            app.display.print(&format!("{} (mode: {})\n", <$test as Test>::name(), mode), None).unwrap();

            app.display.print(&format!("Running {} test case{}...",
                <$test as Test>::cases().len(),
                if <$test as Test>::cases().len() == 1 { "" } else { "s" }), None
            ).unwrap();

            let result = <$test as Test>::run_all(app);

            // Record or compare the results

            // #[cfg(feature = "record")]
            // record(result)?;

            // #[cfg(feature = "compare")]
            // compare(result)?;

            app.display.print("\nDone!\n", Some($crate::display::SUCCESS)).unwrap();
            app.display.frame(true).unwrap();

            // Wait for reboot

            app.wait_for_reboot();
        }
    }
}

// /// Macro that defines a test implementing the `Test` trait and wires it to the entrypoint.
// #[macro_export]
// macro_rules! run_testxxx {
//     ($test_type:ident $test_name:ident { $($body:tt)* }) => {
//         extern crate alloc;

//         use alloc::{format, string::*, vec::*, vec};
//         use anyhow::{anyhow, Result};
//         use n64_specs as specs;
//         use test_suite_common::{
//             Message,
//             result::{TestResult, TestCaseResult},
//             test::{TestNoParams, TestWithParams}
//         };
//         use test_suite_rom::*;

//         fn framebuffer() -> &'static mut $crate::framebuffer::Framebuffer {
//             unsafe { &mut *$crate::FRAMEBUFFER }
//         }

//         #[panic_handler]
//         fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
//             framebuffer().print(&alloc::format!("{}", info), Some($crate::framebuffer::ERROR)).ok();
//             framebuffer().frame(false).ok();

//             $crate::sc64::Sc64::send(Message::Panic).ok();

//             $crate::sc64::Sc64::wait_for_reboot();
//         }

//         struct $test_name;

//         impl $test_type for $test_name {
//             $($body)*
//         }

//         #[unsafe(no_mangle)]
//         extern "C" fn _entrypoint() -> ! {
//             // Setup the global allocator

//             $crate::allocator::configure();

//             // Setup the framebuffer

//             let mut fb = alloc::boxed::Box::new($crate::framebuffer::Framebuffer::new());

//             unsafe { FRAMEBUFFER = &raw mut *fb; }

//             //framebuffer().print(&alloc::format!("Heap: {} / {} bytes", $crate::allocator::used(), $crate::allocator::size()), None).unwrap();

//             // Start the main loop

//             match main_loop() {
//                 Ok(()) => $crate::sc64::Sc64::wait_for_reboot(),
//                 Err(e) => panic!("{e:#}"),
//             }
//         }

//         fn main_loop() -> Result<()> {
//             let (mode, verb) = if cfg!(feature = "record") {
//                 ("record", "Recording")
//             } else {
//                 ("compare", "Comparing")
//             };

//             framebuffer().print(&alloc::format!("{} (mode: {})\n", $test_name::name(), mode), None)?;

//             // Run the test

//             framebuffer().print("Running test...", None)?;

//             let result = $test_name::run_all();

//             // Record or compare the results

//             #[cfg(feature = "record")]
//             record(result)?;

//             #[cfg(feature = "compare")]
//             compare(result)?;

//             Ok(())
//         }

//         /// Record mode: sends the test results to the server.
//         #[cfg(feature = "record")]
//         fn record(result: TestResult) -> Result<()> {
//             framebuffer().print("Sending results over USB...", None)?;

//             $crate::sc64::Sc64::configure()?;
//             $crate::sc64::Sc64::send(Message::TestResult(result))?;

//             framebuffer().print("\nDone!\n", Some($crate::framebuffer::SUCCESS))?;
//             framebuffer().frame(true)
//         }

//         /// Compare mode: compares the test results with the embedded results recorded on hardware.
//         #[cfg(feature = "compare")]
//         fn compare(result: TestResult) -> Result<()> {
//             let reference_result_data = include_bytes!(concat!(
//                 env!("CARGO_MANIFEST_DIR"),
//                 "/../_test_suite_output/",
//                 stringify!($test_name),
//                 ".bin"
//             ));

//             let reference_result: TestResult = postcard::from_bytes(reference_result_data)
//                 .map_err(|e| anyhow!("failed to deserialize embedded result: {e}"))?;

//             let success = result == reference_result;

//             if success {
//                 framebuffer().print("\nSuccess!\n", Some($crate::framebuffer::SUCCESS))?;
//             } else {
//                 framebuffer().print("\nFailure!\n", Some($crate::framebuffer::WARNING))?;

//                 if let Some(diff) = result.first_diff(&reference_result) {
//                     framebuffer().print(&diff, Some($crate::framebuffer::ERROR))?;
//                 }
//             }

//             framebuffer().frame(success)
//         }
//     };
// }
