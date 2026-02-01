use std::path::Path;

use color_eyre::eyre::WrapErr;
use n64::{breakpoints::Breakpoint, cart::Cart, cpu::CPU};

use crate::tui::{App, RunMode, State};

mod tui;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    tui::logger::init(log::LevelFilter::Trace);

    let cart = Cart::load(Path::new("sm.n64")).wrap_err("load ROM")?;

    let mut cpu = CPU::new(cart);
    cpu.skip_ipl();

    cpu.breakpoints.add(Breakpoint::Address(0x8000_0130)); //a4000040

    let mut app = App {
        state: State::Running(RunMode::Loop),
        cpu,
        logs: Vec::new(),
    };

    // Main screen so last frame stays visible after exit/panic

    let options = ratatui::TerminalOptions {
        viewport: ratatui::Viewport::Fullscreen,
    };

    let mut terminal = ratatui::try_init_with_options(options).wrap_err("TUI init")?;

    terminal.clear().wrap_err("TUI clear")?;

    let result = app.run(&mut terminal);

    ratatui::try_restore().wrap_err("TUI restore")?;
    result?;

    Ok(())
}
