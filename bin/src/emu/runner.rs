use std::thread::{self, JoinHandle};

use crossbeam::channel::{Receiver, Sender, TryRecvError, unbounded};
use n64::{
    instructions::{Opcode, decode},
    system::System,
};

use crate::{
    emu::{command::Command, event::Event},
    ui::instructions::InstructionData,
};

#[derive(Debug)]
pub enum RunMode {
    Running,
    Paused,
    Exited,
}

pub struct State {
    pub system: Option<System>,
    pub run_mode: RunMode,
}

pub struct Runner {
    thread: JoinHandle<()>,
    command_tx: Sender<Command>,
    pub event_rx: Receiver<Event>, // TODO pub
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
                    run_mode: RunMode::Running,
                };

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

                    // Update the system

                    if let Some(system) = &mut state.system {
                        system.step();

                        let instructions = (system.cpu.regs.pc..system.cpu.regs.pc + 16 * 4)
                            .step_by(4)
                            .map(|addr| {
                                let instruction = system.read(addr);

                                let opcode = Opcode(instruction);
                                let handler = decode(opcode);
                                let disassembly = handler.disassemble(system, opcode);

                                InstructionData {
                                    address: addr,
                                    disassembly,
                                }
                            })
                            .collect();

                        let mut memory = vec![];

                        for i in 0..64 {
                            memory.push(system.read(i));
                        }

                        event_tx
                            .send(Event::Update {
                                instructions,
                                cpu_regs: system.cpu.regs.clone(),
                                memory,
                            })
                            .unwrap();
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

    pub fn send_command(&self, command: Command) {
        self.command_tx.send(command).unwrap();
    }
}
