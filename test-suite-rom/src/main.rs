#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(asm_experimental_arch)]
#![feature(used_with_arg)]

#[cfg(not(any(feature = "record", feature = "replay")))]
compile_error!("must enable either feature \"record\" or \"replay\"");

#[cfg(all(feature = "record", feature = "replay"))]
compile_error!("features \"record\" and \"replay\" cannot be both enabled");

mod allocator;
mod app;
mod display;
mod io;
mod isviewer;
mod program;
mod sc64;
mod test;
mod tests;

//#[cfg(feature = "replay")]
mod comparator;

extern crate alloc;

use alloc::format;
use test_suite_common::Message;

use crate::{
    app::App,
    display::{ERROR, TextStyle},
};

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

#[unsafe(no_mangle)]
extern "C" fn _entrypoint() -> ! {
    let app = init_app().expect("failed to initialize app");

    app.run::<tests::CurrentTest>().expect("failed to run app");

    app.wait_for_reboot()
}

/// Registers a test.
///
/// All the tests must be registered with this macro.
///
/// The macro itself doesn't really do anything special, but the build script specifically looks for it when listing the available tests.
#[macro_export]
macro_rules! register_test {
    ($test:ident) => {
        pub struct $test;
    };
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    // Prevent recursive panics

    static mut PANICKING: bool = false;

    unsafe {
        if PANICKING {
            loop {
                core::hint::spin_loop();
            }
        }

        PANICKING = true;
    }

    // Notify

    app()
        .print(&format!("{}", info), Some(TextStyle::with_color(ERROR)))
        .ok();

    app().send(Message::Panic, true).ok();

    // Wait for reboot

    app().wait_for_reboot()
}
