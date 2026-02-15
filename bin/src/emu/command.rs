use std::path::PathBuf;

use n64::{cart::Cart, system::System};

use crate::{
    emu::runner::{RunMode, State},
    ui::UiSettings,
};

pub enum Command {
    SetSettings { settings: UiSettings },
    LoadRom { path: PathBuf },
    Pause,
    Resume,
    Step,
    Exit,
}

impl Command {
    pub fn handle(&self, state: &mut State) {
        match self {
            Command::SetSettings { settings } => {
                state.ui_settings = settings.clone();
            }

            Command::LoadRom { path } => {
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
                state.run_mode = RunMode::Paused {
                    step_requested: false,
                };
            }

            Command::Resume => {
                state.run_mode = RunMode::Running;
            }

            Command::Step => {
                if let RunMode::Paused { step_requested } = &mut state.run_mode {
                    *step_requested = true;
                }
            }

            Command::Exit => {
                state.run_mode = RunMode::Exited;
            }
        }
    }
}
