#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

#[cfg(not(any(feature = "collect", feature = "compare")))]
compile_error!("must enable either feature \"collect\" or \"compare\"");

#[cfg(all(feature = "collect", feature = "compare"))]
compile_error!("features \"collect\" and \"compare\" are mutually exclusive");

extern crate alloc;

pub mod allocator;
pub mod framebuffer;
mod isviewer;
pub mod sc64;

// TODO clean up
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

// TODO clean up
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::isviewer::write_fmt(format_args!($($arg)*)));
}

// TODO clean up
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Macro that defines a test implementing the `Test` trait and wires it to the entrypoint.
#[macro_export]
macro_rules! define_test {
    ($test:ident { $($body:tt)* }) => {
        extern crate alloc;

        use test_suite_common::*;
        use test_suite_rom::*;

        pub struct $test;

        impl Test for $test {
            $($body)*
        }

        #[unsafe(no_mangle)]
        extern "C" fn _entrypoint() -> ! {
            $crate::allocator::configure();

            main_loop::<$test>()
        }

        pub fn main_loop<T: Test>() -> ! {
            $crate::framebuffer::Framebuffer::configure();
            $crate::sc64::Sc64::configure();

            let result = T::run();

            #[cfg(feature = "collect")]
            collect(result);

            #[cfg(feature = "compare")]
            compare(result);

            loop {
                core::hint::spin_loop();
            }
        }

        /// Collect mode: sends the test results to the server.
        #[cfg(feature = "collect")]
        fn collect(result: TestResult) {
            $crate::sc64::Sc64::send(Message::TestResult(result));
        }

        /// Compare mode: compares the test results with the embedded results collected on hardware.
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

            use $crate::framebuffer::*;
            Framebuffer::fill(if success { GREEN } else { RED });
        }
    };
}
