#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(asm_experimental_arch)]
#![feature(naked_functions)]
#![feature(used_with_arg)]

#[cfg(not(any(feature = "record", feature = "replay")))]
compile_error!("must enable either feature \"record\" or \"replay\"");

#[cfg(all(feature = "record", feature = "replay"))]
compile_error!("features \"record\" and \"replay\" cannot be both enabled");

mod allocator;
mod app;
mod display;
mod exceptions;
mod io;
mod isviewer;
mod program;
mod sc64;
mod test;
mod tests;

//#[cfg(feature = "replay")]
mod comparator;

// TODO
// - RAM reg mirroring

extern crate alloc;

use alloc::format;
use test_suite_common::Message;

use crate::{
    app::App,
    display::{ERROR, TextStyle},
};

// Global app instance (static to access it from the global panic handler with `app()`)
static mut APP: *mut App = core::ptr::null_mut();

pub fn app() -> Option<&'static mut App> {
    unsafe { if APP.is_null() { None } else { Some(&mut *APP) } }
}

pub fn init_app() -> anyhow::Result<&'static mut App> {
    crate::allocator::configure();

    let app_boxed = alloc::boxed::Box::new(App::new()?);

    unsafe {
        APP = &raw mut *alloc::boxed::Box::into_raw(app_boxed);

        Ok(&mut *APP)
    }
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
    // Intercept recursive panics

    static mut PANICKING: bool = false;

    unsafe {
        if PANICKING {
            loop {
                core::hint::spin_loop();
            }
        }

        PANICKING = true;
    }

    if let Some(app) = app() {
        // Notify

        app.print(&format!("{}", info), Some(TextStyle::with_color(ERROR)))
            .ok();

        app.send(Message::Panic, true).ok();

        // Wait for reboot

        app.wait_for_reboot()
    } else {
        // The IS-VIEWER output doesn't require the app to be initialized,
        // so this is our last resort to communicate what went wrong

        isviewer::write(&format!("{}", info));

        loop {
            core::hint::spin_loop();
        }
    }
}
