#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(asm_experimental_arch)]
#![feature(used_with_arg)]

#[cfg(not(any(feature = "record", feature = "replay")))]
compile_error!("must enable either feature \"record\" or \"replay\"");

#[cfg(all(feature = "record", feature = "replay"))]
compile_error!("features \"record\" and \"replay\" are mutually exclusive");

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

#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    app()
        .print(&format!("{}", info), Some(TextStyle::with_color(ERROR)))
        .ok();

    app().display.frame(false).ok();

    app().send(Message::Panic, true).ok();

    app().wait_for_reboot()
}

#[unsafe(no_mangle)]
extern "C" fn _entrypoint() -> ! {
    let app = init_app().expect("failed to initialize app");

    app.run::<tests::CurrentTest>().expect("failed to run test");

    app.wait_for_reboot()
}
