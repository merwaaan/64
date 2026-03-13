use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use n64_core::{cart::Cart, controller::Button, system::System};

use crate::{
    core_thread::{CoreThreadState, CoreThreadStatus},
    event::Event,
    ui::{Data, Status, widgets::WidgetId},
};

#[derive(Debug)]
pub enum ControllerInput {
    PressButton(Button),
    ReleaseButton(Button),
    AxisChanged(bool, f32),
}

/// Commands sent from the UI to the core thread
#[derive(Debug)]
pub enum Command {
    LoadRom(PathBuf),
    Pause,
    Resume,
    Step,
    RequestData(WidgetId, HashSet<Data>),
    AddBreakpoint(u32),
    ToggleBreakpoint(u32),
    RemoveBreakpoint(u32),
    ControllerInput(ControllerInput),
}

impl Command {
    pub fn handle(
        &self,
        state: &mut CoreThreadState,
        requested_data: &mut HashMap<WidgetId, HashSet<Data>>,
    ) -> Vec<Event> {
        let mut events = Vec::new();

        match self {
            Command::LoadRom(path) => {
                log::info!("Loading ROM {}", path.display());

                match Cart::load(path) {
                    Ok(cart) => {
                        let system = System::with_cart(cart);

                        events.push(Event::Breakpoints(system.breakpoints().clone()));

                        state.system = Some(system);
                    }
                    Err(e) => {
                        log::error!("Failed to load cart: {}", e);
                    }
                }
            }
            Command::Pause => {
                state.status = CoreThreadStatus::Paused { step: false };

                events.push(Event::Status(Status::Paused));
            }
            Command::Resume => {
                state.status = CoreThreadStatus::Running;

                events.push(Event::Status(Status::Running));
            }
            Command::Step => {
                state.status = CoreThreadStatus::Paused { step: true };

                events.push(Event::Status(Status::Paused));
            }
            Command::RequestData(id, data) => {
                if data.is_empty() {
                    requested_data.remove(id);
                } else {
                    requested_data.insert(*id, data.clone());
                }
            }
            Command::AddBreakpoint(breakpoint) => {
                if let Some(system) = &mut state.system {
                    system.add_breakpoint(*breakpoint);

                    events.push(Event::Breakpoints(system.breakpoints().clone()));
                }
            }
            Command::RemoveBreakpoint(breakpoint) => {
                if let Some(system) = &mut state.system {
                    system.remove_breakpoint(*breakpoint);

                    events.push(Event::Breakpoints(system.breakpoints().clone()));
                }
            }
            Command::ToggleBreakpoint(address) => {
                if let Some(system) = &mut state.system {
                    system.toggle_breakpoint(*address);

                    events.push(Event::Breakpoints(system.breakpoints().clone()));
                }
            }
            Command::ControllerInput(input) => {
                if let Some(system) = &mut state.system {
                    match input {
                        ControllerInput::PressButton(button) => {
                            system.controllers[0].press(*button);
                        }
                        ControllerInput::ReleaseButton(button) => {
                            system.controllers[0].release(*button);
                        }
                        ControllerInput::AxisChanged(axis, value) => {
                            system.controllers[0].set_axis(*axis, *value);
                        }
                    }
                }
            }
        }

        events
    }
}
