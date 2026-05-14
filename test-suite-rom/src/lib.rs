#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
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

// Global app instance (static to access it from the panic handler)
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

            app.display.print(&format!("{} {} test case{}...",
                verb,
                <$test as Test>::cases().len(),
                if <$test as Test>::cases().len() == 1 { "" } else { "s" }), None
            ).unwrap();

            let result = <$test as Test>::run_all(app);

            app.display.print("\nDone!\n", Some($crate::display::SUCCESS)).unwrap();
            app.display.frame(true).unwrap();

            // Wait for reboot

            app.wait_for_reboot();
        }
    }
}
