use std::path::PathBuf;

use n64_core::{cart::Cart, system::System};

use crate::{
    emu::{
        core_thread::{CoreThread, CoreThreadState, CoreThreadStatus},
        event::Event,
    },
    ui::{SettingUpdate, Status},
};

pub enum Command {
    LoadRom(PathBuf),
    Pause,
    Resume,
    Step,
    SetSetting(SettingUpdate),
    AddBreakpoint(u32),
    ToggleBreakpoint(u32),
    RemoveBreakpoint(u32),
}

impl Command {
    pub fn handle(&self, state: &mut CoreThreadState) -> Vec<Event> {
        let mut events = Vec::new();

        match self {
            Command::LoadRom(path) => {
                log::info!("Loading ROM {}", path.display());

                match Cart::load(path) {
                    Ok(cart) => {
                        let mut system = System::new(cart);
                        system.skip_ipl(); // TODO internalize

                        events.push(Event::BreakpointsUpdate(system.breakpoints().clone()));

                        state.system = Some(system);

                        events.extend(CoreThread::create_update_events(state));
                    }
                    Err(e) => {
                        log::error!("Failed to load cart: {}", e);
                    }
                }
            }
            Command::Pause => {
                state.status = CoreThreadStatus::Paused { step: false };

                events.push(Event::StatusUpdate(Status::Paused));
            }
            Command::Resume => {
                state.status = CoreThreadStatus::Running;

                events.push(Event::StatusUpdate(Status::Running));
            }
            Command::Step => {
                state.status = CoreThreadStatus::Paused { step: true };

                events.push(Event::StatusUpdate(Status::Paused));
            }
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

                events.extend(CoreThread::create_update_events(state));
            }
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
