use std::thread::{self, JoinHandle};

use crossbeam::channel::{Receiver, Sender, TryRecvError, unbounded};
use n64::{
    instructions::{Opcode, decode},
    system::System,
};

use crate::{
    emu::{command::Command, event::Event},
    ui::{UiSettings, instructions::InstructionData},
};

#[derive(Debug)]
pub enum RunMode {
    Running,
    Paused { step_requested: bool },
    Exited,
}

pub struct State {
    pub system: Option<System>,
    pub run_mode: RunMode,
    pub ui_settings: UiSettings,
}

pub struct Runner {
    thread: JoinHandle<()>,
    command_tx: Sender<Command>,
    event_rx: Receiver<Event>,
}

impl Runner {
    pub fn new() -> Self {
        // Setup the core thread

        let (command_tx, command_rx) = unbounded::<Command>();
        let (event_tx, event_rx) = unbounded::<Event>();

        let thread = thread::Builder::new()
            .name("Core".to_string())
            .spawn(move || {
                let mut state: State = State {
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
                                command.handle(&mut state);
                            }
                            Err(TryRecvError::Empty) => {
                                break;
                            }
                            Err(error) => {
                                panic!("Runner channel error: {:?}", error);
                            }
                        }
                    }

                    // Update the system (when running, or one step when paused and step requested)

                    let do_step = matches!(state.run_mode, RunMode::Running)
                        || matches!(
                            state.run_mode,
                            RunMode::Paused {
                                step_requested: true
                            }
                        );

                    if do_step {
                        if let Some(system) = &mut state.system {
                            system.step();

                            if let RunMode::Paused { step_requested } = &mut state.run_mode {
                                *step_requested = false;
                            }

                            xxx += 1;

                            let send_event = xxx % 1000 == 0;

                            if send_event {
                                let instructions =
                                    state.ui_settings.instructions.as_ref().map(|settings| {
                                        (settings.address
                                            ..settings.address + settings.rows as u32 * 4)
                                            .step_by(4)
                                            .map(|addr| {
                                                let instruction = system.read(addr);
                                                let opcode = Opcode(instruction);
                                                let handler = decode(opcode);
                                                let disassembly =
                                                    handler.disassemble(system, opcode);
                                                InstructionData {
                                                    address: addr,
                                                    disassembly,
                                                }
                                            })
                                            .collect()
                                    });

                                let cpu_regs =
                                    state.ui_settings.cpu_regs.map(|_| system.cpu.regs.clone()); // TODO check bool

                                let memory = state.ui_settings.memory.as_ref().map(|settings| {
                                    (0..settings.rows * 4)
                                        .map(|i| system.read(settings.address + (i as u32) * 4))
                                        .collect()
                                });

                                event_tx
                                    .send(Event::Update {
                                        instructions,
                                        cpu_regs,
                                        memory,
                                    })
                                    .unwrap();
                            }
                        }
                    }

                    // Exit?

                    if matches!(state.run_mode, RunMode::Exited) {
                        break;
                    }
                }
            })
            .unwrap();

        Self {
            thread,
            command_tx,
            event_rx,
        }
    }

    /// Sends a command to the core thread.
    pub fn send_command(&self, command: Command) {
        self.command_tx.send(command).unwrap();
    }

    /// Receives one event if available.
    pub fn poll_event(&self) -> Option<Event> {
        self.event_rx.try_recv().ok()
    }
}
