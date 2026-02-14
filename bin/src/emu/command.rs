use std::path::PathBuf;

use n64::{cart::Cart, system::System};

use crate::emu::runner::{RunMode, State};

#[derive(Debug)]
pub enum Command {
    LoadRom { path: PathBuf },
    Pause,
    Resume,
    Exit,
}

impl Command {
    pub fn handle(&self, state: &mut State) {
        match self {
            Command::LoadRom { path } => {
                log::info!("Loading ROM {}", path.display());

                let cart = match Cart::load(path) {
                    Ok(c) => c,
                    Err(e) => {
                        log::error!("Failed to load ROM: {}", e);
                        return;
                    }
                };

                let mut system = System::new(cart, None, None);
                system.skip_ipl(); // TODO internalize

                state.system = Some(system);
            }
            Command::Pause => {
                log::info!("Pausing emulator");
                state.run_mode = RunMode::Paused;
            }
            Command::Resume => {
                log::info!("Resuming emulator");
                state.run_mode = RunMode::Running;
            }
            Command::Exit => {
                log::info!("Exiting emulator");
                state.run_mode = RunMode::Exited;
            }
        }
    }
}
