#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(asm_experimental_arch)]
#![feature(used_with_arg)]

#[cfg(not(any(feature = "record", feature = "replay")))]
compile_error!("must enable either feature \"record\" or \"replay\"");

#[cfg(all(feature = "record", feature = "replay"))]
compile_error!("features \"record\" and \"replay\" are mutually exclusive");

pub mod allocator;
pub mod app;
pub mod display;
pub mod io;
pub mod isviewer;
pub mod program;
pub mod sc64;
pub mod test;
//#[cfg(feature = "replay")]
pub mod comparator;

extern crate alloc;

use crate::app::App;

// Global app instance (static to access it from the panic handler)
static mut APP: *mut App = core::ptr::null_mut();

pub fn app() -> &'static mut App {
    unsafe { &mut *APP }
}

pub fn init_app() -> anyhow::Result<&'static mut App> {
    crate::allocator::configure();

    let app_boxed = alloc::boxed::Box::new(App::new()?);

    unsafe {
        APP = &raw mut *alloc::boxed::Box::into_raw(app_boxed);
    }

    Ok(app())
}

#[macro_export]
macro_rules! run_test {
    {$test:ident} => {
        extern crate alloc;

        // Import useful types
        use alloc::{format, string::*, vec::Vec};
        use anyhow::Context;
        use arbitrary_int::prelude::*;
        use anyhow::Result;
        use core::arch::asm;
        use n64_specs as specs;
        use test_suite_common::*;
        use test_suite_rom::{*, program::*};
        use crate::{app::App, test::{Test, TestError}};

        // Setup the panic handler

        #[panic_handler]
        fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
            $crate::app().print(
                &alloc::format!("{}", info),
                Some($crate::display::TextStyle::with_color($crate::display::ERROR))
            ).ok();

            $crate::app().display.frame(false).ok();

            $crate::app().send(Message::Panic, true).ok();

            $crate::app().wait_for_reboot()
        }

        // Define the test struct so that each test only requires an `impl Test`

        struct $test;

        // Define the entry point

        #[unsafe(no_mangle)]
        extern "C" fn _entrypoint() -> ! {
            let mut app = $crate::init_app().expect("failed to initialize app");

            app.run::<$test>().expect("failed to run test");

            app.wait_for_reboot();
        }
    }
}
