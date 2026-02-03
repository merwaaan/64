use std::path::Path;

use clap::Parser;
use color_eyre::eyre::WrapErr;
use n64::{breakpoints::Breakpoint, cart::Cart, system::System};

use crate::tui::{App, RunMode, State};

mod tui;

#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"), about = "N64 emulator debugger")]
struct Args {
    /// Path to the ROM file (.z64, .n64)
    #[arg()]
    rom: String,

    /// Breakpoint addresses (decimal or hex with 0x prefix)
    #[arg(short, long, value_parser = parse_addr)]
    breakpoint: Vec<u32>,

    /// Start step for logging
    #[arg(long)]
    log_from: Option<usize>,

    /// End step for logging
    #[arg(long)]
    log_to: Option<usize>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    tui::logger::init(log::LevelFilter::Trace);

    let args = Args::parse();

    // Setup the system

    let cart = Cart::load(Path::new(&args.rom)).wrap_err("load ROM")?;

    let mut system = System::new(cart, args.log_from, args.log_to);
    system.skip_ipl();

    // start: 0xa4000040
    for addr in &args.breakpoint {
        system.breakpoints.add(Breakpoint::Address(*addr));
    }

    // Setup the TUI

    let mut app = App {
        state: State::Running(RunMode::Loop),
        system,
        logs: Vec::new(),
    };

    let mut terminal = ratatui::try_init().wrap_err("TUI init")?;

    app.run(&mut terminal)?;

    Ok(())
}

fn parse_addr(s: &str) -> Result<u32, String> {
    let s = s.trim();

    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).map_err(|e| e.to_string())
    } else {
        s.parse::<u32>().map_err(|e| e.to_string())
    }
}
