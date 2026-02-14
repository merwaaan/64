use clap::Parser;
use env_logger::Env;

use crate::ui::Ui;

mod emu;
mod tui;
mod ui;

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

fn main() -> iced::Result {
    env_logger::Builder::from_env(Env::default().default_filter_or("bin=info")).init();

    let args = Args::parse();

    iced::application(Ui::new, Ui::update, Ui::view)
        .subscription(Ui::subscribe)
        .run()
}

fn parse_addr(s: &str) -> Result<u32, String> {
    let s = s.trim();

    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).map_err(|e| e.to_string())
    } else {
        s.parse::<u32>().map_err(|e| e.to_string())
    }
}
