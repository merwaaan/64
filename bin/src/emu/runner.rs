use std::thread::{self, JoinHandle};

use crossbeam::channel::{Receiver, Sender, TryRecvError, unbounded};
use n64::{
    instructions::{Disassembly, Opcode, decode},
    system::System,
    vi::Vi,
};

use crate::{
    emu::{command::Command, event::Event},
    ui::{
        framebuffer::FramebufferUpdate,
        instructions::{InstructionAddress, InstructionData, InstructionsSettings},
        memory::{MemorySettings, MemoryUpdate},
        registers::RegistersUpdate,
    },
};

#[derive(Debug)]
pub enum RunMode {
    Running,
    Paused { step_requested: bool },
    Exited,
}

#[derive(Default)]
pub struct UiSettings {
    pub instructions: Option<InstructionsSettings>,
    pub registers: Option<()>,
    pub memory: Option<MemorySettings>,
    pub framebuffer: Option<()>,
}

pub struct RunnerState {
    pub system: Option<System>,
    pub run_mode: RunMode,
    pub ui_settings: UiSettings,
}

pub struct Runner {
    thread: JoinHandle<()>,
    command_tx: Sender<Command>,
    event_rx: Receiver<Event>,
}

const UPDATE_INTERVAL: usize = 100_000;

impl Runner {
    pub fn new() -> Self {
        // Setup the core thread

        let (command_tx, command_rx) = unbounded::<Command>();
        let (event_tx, event_rx) = unbounded::<Event>();

        let thread = thread::Builder::new()
            .name("Core".to_string())
            .spawn(move || {
                let mut state: RunnerState = RunnerState {
                    system: None,
                    run_mode: RunMode::Paused {
                        step_requested: false,
                    },
                    ui_settings: UiSettings::default(),
                };

                let mut xxx = 0;

                loop {
                    // Handle commands

                    loop {
                        match command_rx.try_recv() {
                            Ok(command) => {
                                let events = command.handle(&mut state);

                                for event in events {
                                    event_tx.send(event).unwrap();
                                }
                            }
                            Err(TryRecvError::Empty) => {
                                break;
                            }
                            Err(error) => {
                                panic!("Runner channel error: {:?}", error);
                            }
                        }
                    }

                    // Update the system

                    let step = matches!(state.run_mode, RunMode::Running)
                        || matches!(
                            state.run_mode,
                            RunMode::Paused {
                                step_requested: true
                            }
                        );

                    if step && let Some(system) = &mut state.system {
                        let mut send_update = false;

                        let breakpoint_hit = system.step();

                        if breakpoint_hit {
                            state.run_mode = RunMode::Paused {
                                step_requested: false,
                            };

                            event_tx
                                .send(Event::Pause)
                                .expect("Failed to send pause event");

                            send_update = true;
                        } else {
                            if let RunMode::Paused { step_requested } = &mut state.run_mode {
                                *step_requested = false;
                                send_update = true;
                            }

                            xxx += 1;

                            if xxx % UPDATE_INTERVAL == 0 {
                                send_update = true;
                            }
                        }

                        if send_update {
                            for event in Runner::update_events(&state) {
                                event_tx.send(event).unwrap();
                            }
                        }
                    }

                    // Exit?

                    if matches!(state.run_mode, RunMode::Exited) {
                        break;
                    }
                }

                log::info!("Core thread exited");
            })
            .expect("Failed to spawn core thread");

        Self {
            thread,
            command_tx,
            event_rx,
        }
    }

    /// Sends a command to the core thread.
    pub fn send_command(&self, command: Command) {
        self.command_tx
            .send(command)
            .expect("Failed to send command");
    }

    /// Receives an event from the core thread if available.
    pub fn poll_event(&self) -> Option<Event> {
        self.event_rx.try_recv().ok()
    }

    pub fn update_events(state: &RunnerState) -> Vec<Event> {
        let mut events = Vec::new();

        if let Some(system) = &state.system {
            if let Some(instructions_settings) = state.ui_settings.instructions.as_ref() {
                let base_address = match instructions_settings.base_address {
                    InstructionAddress::Pc => system.cpu.regs.pc,
                    InstructionAddress::Address(addr) => addr,
                };

                let instructions = (base_address
                    ..base_address + instructions_settings.rows as u32 * 4)
                    .step_by(4)
                    .map(|addr| {
                        let instruction = system.read(addr);

                        let opcode = Opcode(instruction);
                        let handler = decode(opcode);

                        if let Some(handler) = handler {
                            let disassembly = handler.disassemble(system, opcode);

                            InstructionData {
                                address: addr,
                                disassembly,
                            }
                        } else {
                            InstructionData {
                                address: addr,
                                disassembly: Disassembly::new("<UNKNOWN>".to_string()),
                            }
                        }
                    })
                    .collect();

                events.push(Event::InstructionsUpdate(instructions));
            }

            if state.ui_settings.registers.is_some() {
                let registers = RegistersUpdate {
                    cpu_regs: system.cpu.regs,
                    cop0_regs: system.cop0.regs,
                };

                events.push(Event::RegistersUpdate(registers));
            }

            if let Some(memory_settings) = state.ui_settings.memory.as_ref() {
                let base_address = memory_settings.address & !0xF;

                let data = (0..memory_settings.rows * 16)
                    .map(|i| system.read(base_address + i as u32))
                    .collect();

                events.push(Event::MemoryUpdate(MemoryUpdate { base_address, data }));
            }

            if let Some(()) = state.ui_settings.framebuffer.as_ref() {
                let (data, width, height) = Vi::extract_framebuffer(system);

                events.push(Event::FramebufferUpdate(FramebufferUpdate {
                    width,
                    height,
                    data,
                }));
            }

            // TODO conditional
            events.push(Event::MiUpdate(system.map.mi));
            events.push(Event::ViUpdate(system.map.vi));
        }

        events
    }
}
