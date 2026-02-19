use std::path::PathBuf;

use n64::{cart::Cart, system::System};

use crate::{
    emu::{
        event::Event,
        runner::{RunMode, Runner, RunnerState},
    },
    ui::SettingUpdate,
};

pub enum Command {
    SetSetting(SettingUpdate),
    AddBreakpoint(u32),
    ToggleBreakpoint(u32),
    RemoveBreakpoint(u32),
    LoadRom(PathBuf),
    Pause,
    Resume,
    Step,
    //Exit,
}

impl Command {
    pub fn handle(&self, state: &mut RunnerState) -> Vec<Event> {
        let mut events = Vec::new();

        match self {
            Command::SetSetting(setting) => {
                match setting {
                    SettingUpdate::Instructions(settings) => {
                        state.ui_settings.instructions = *settings;
                    }
                    SettingUpdate::Memory(settings) => {
                        state.ui_settings.memory = *settings;
                    }
                    SettingUpdate::Registers(settings) => {
                        state.ui_settings.registers = *settings;
                    }
                    SettingUpdate::Framebuffer(settings) => {
                        state.ui_settings.framebuffer = *settings;
                    }
                }

                events.extend(Runner::update_events(state));
            }

            Command::LoadRom(path) => {
                match Cart::load(path) {
                    Ok(cart) => {
                        let mut system = System::new(cart);
                        system.skip_ipl(); // TODO internalize

                        events.push(Event::BreakpointsUpdate(system.breakpoints().clone()));

                        state.system = Some(system);

                        events.extend(Runner::update_events(state));
                    }
                    Err(e) => {
                        log::error!("Failed to load cart: {}", e);
                    }
                }
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

            // Command::Exit => {
            //     state.run_mode = RunMode::Exited;
            // }
            Command::AddBreakpoint(breakpoint) => {
                if let Some(system) = &mut state.system {
                    system.add_breakpoint(*breakpoint);

                    events.push(Event::BreakpointsUpdate(system.breakpoints().clone()));
                }
            }

            Command::RemoveBreakpoint(breakpoint) => {
                if let Some(system) = &mut state.system {
                    system.remove_breakpoint(*breakpoint);

                    events.push(Event::BreakpointsUpdate(system.breakpoints().clone()));
                }
            }

            Command::ToggleBreakpoint(address) => {
                if let Some(system) = &mut state.system {
                    system.toggle_breakpoint(*address);

                    events.push(Event::BreakpointsUpdate(system.breakpoints().clone()));
                }
            }
        }

        events
    }
}
