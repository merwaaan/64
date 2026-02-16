use clap::Parser;
use egui::ViewportBuilder;
use env_logger::Env;

use crate::ui::Ui;

mod emu;
mod ui;

#[derive(clap::Parser)]
#[command(name = env!("CARGO_PKG_NAME"), about = "N64 emulator debugger")]
pub struct Args {
    /// Path to the ROM file (.z64, .n64)
    #[arg()]
    rom: Option<String>,
}

fn main() -> eframe::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("bin=info,n64=info")).init();

    let args = Args::parse();

    eframe::run_native(
        "N64 Debugger",
        eframe::NativeOptions {
            viewport: ViewportBuilder::default()
                .with_inner_size([2000.0, 1500.0])
                .with_drag_and_drop(true),
            ..Default::default()
        },
        Box::new(|_cc| Ok(Box::new(Ui::new(&args)))),
    )
}
