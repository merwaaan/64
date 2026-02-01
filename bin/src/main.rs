use std::path::Path;

use color_eyre::eyre::WrapErr;
use n64::{breakpoints::Breakpoint, cart::Cart, cpu::CPU};

use crate::tui::{App, State};

mod tui;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    tui::logger::init(log::LevelFilter::Trace);

    let cart = Cart::load(Path::new("sm.n64")).wrap_err("load ROM")?;

    let mut cpu = CPU::new();
    cpu.skip_ipl(&cart);

    cpu.breakpoints.add(Breakpoint::Address(0xA40004B8));

    let mut app = App {
        state: State::Paused,
        cpu,
        cart,
        logs: Vec::new(),
    };

    // TODO move to tui
    ratatui::run(|terminal| app.run(terminal)).wrap_err("TUI run")?;

    Ok(())
}
