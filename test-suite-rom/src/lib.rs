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

        use n64_specs as specs;
        use test_suite_common::*;
        use test_suite_rom::*;

        static mut FRAMEBUFFER: *mut $crate::framebuffer::Framebuffer = core::ptr::null_mut();

        fn framebuffer() -> &'static mut $crate::framebuffer::Framebuffer {
            unsafe { &mut *FRAMEBUFFER }
        }

        #[panic_handler]
        fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
            framebuffer().print(&alloc::format!("{}", info), Some($crate::framebuffer::ERROR));
            framebuffer().frame(false);

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

            main_loop::<$test>()
        }

        fn main_loop<T: Test>() -> ! {
            let (mode, verb) = if cfg!(feature = "record") {
                ("record", "Recording")
            } else {
                ("compare", "Comparing")
            };

            framebuffer().print(&alloc::format!("{} (mode: {})\n", T::name(), mode), None);

            // Run the test

            framebuffer().print(&alloc::format!("{} {} test case{}...", verb, T::cases().len(), if T::cases().len() == 1 { "" } else { "s" }), None);

            let result = T::run();

            // Record or compare the results

            #[cfg(feature = "record")]
            record(result);

            #[cfg(feature = "compare")]
            compare(result);

            // We're done, wait for the SC64 to reboot

            framebuffer().print("Done", None);
            framebuffer().frame(true);

            $crate::sc64::Sc64::wait_for_reboot();
        }

        /// Record mode: sends the test results to the server.
        #[cfg(feature = "record")]
        fn record(result: TestResult) {
            framebuffer().print("Sending results to server...", None);

            $crate::sc64::Sc64::configure();
            $crate::sc64::Sc64::send(Message::TestResult(result));
        }

        /// Compare mode: compares the test results with the embedded results recorded on hardware.
        #[cfg(feature = "compare")]
        fn compare(result: TestResult) {

            let reference_result_data = include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../_test_suite_output/",
                stringify!($test),
                ".bin"
            ));

            let reference_result: TestResult = postcard::from_bytes(reference_result_data)
                .expect("failed to deserialize reference result");

            let success = result == reference_result;

            framebuffer().fill(if success { $crate::framebuffer::SUCCESS } else { $crate::framebuffer::ERROR });
        }
    };
}

pub fn reg_mut_ptr(offset: u32) -> *mut u32 {
    (n64_specs::map::Segment::KSEG1 as u32 | offset) as *mut u32
}
