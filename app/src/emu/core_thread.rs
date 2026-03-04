use std::panic::AssertUnwindSafe;
use std::thread::{self, JoinHandle};

use crossbeam::channel::{Receiver, Sender, TryRecvError, unbounded};
use n64_core::{
    instructions::{Disassembly, Opcode, decode},
    system::System,
    vi::Vi,
};

use crate::ui::Status;
use crate::{
    emu::{command::Command, event::Event},
    ui::{
        framebuffer::FramebufferUpdate,
        instructions::{InstructionAddress, InstructionData, InstructionsSettings},
        memory::{MemorySettings, MemoryUpdate},
        registers::RegistersUpdate,
    },
};

#[derive(Default)]
pub struct UiSettings {
    pub instructions: Option<InstructionsSettings>,
    pub registers: Option<()>,
    pub memory: Option<MemorySettings>,
    pub framebuffer: Option<()>,
}

#[derive(Debug, Clone, Copy)]
pub enum CoreThreadStatus {
    Running,
    Paused { step: bool },
}

impl From<CoreThreadStatus> for Status {
    fn from(status: CoreThreadStatus) -> Status {
        match status {
            CoreThreadStatus::Running => Status::Running,
            CoreThreadStatus::Paused { .. } => Status::Paused,
        }
    }
}

pub struct CoreThreadState {
    pub system: Option<System>,
    pub status: CoreThreadStatus,
    pub ui_settings: UiSettings,
}

pub struct CoreThread {
    thread: JoinHandle<()>,
    command_tx: Sender<Command>,
    event_rx: Receiver<Event>,
}

const UPDATE_INTERVAL: usize = 100_000;

impl CoreThread {
    pub fn new() -> Self {
        // Setup the core thread

        let (command_tx, command_rx) = unbounded::<Command>();
        let (event_tx, event_rx) = unbounded::<Event>();

        let thread = thread::Builder::new()
            .name("Core".to_string())
            .spawn(move || {
                // Thread loop;
                // - create a state
                // - run the emulation loop
                // - restart if something fails

                loop {
                    // Run the core loop

                    log::debug!("Core loop started");

                    let mut state = CoreThreadState {
                        system: None,
                        status: CoreThreadStatus::Paused { step: false },
                        ui_settings: UiSettings::default(),
                    };

                    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                        Self::run_core_loop(&mut state, &command_rx, &event_tx);
                    }));

                    // Restart the core loop if it panicked

                    if result.is_err() {
                        log::warn!("Core loop panicked");

                        // Notify the UI

                        event_tx
                            .send(Event::StatusUpdate(Status::Panicked(format!(
                                "{:?}",
                                result.unwrap_err()
                            ))))
                            .inspect_err(|error| {
                                log::error!("Failed to send status update: {:?}", error)
                            })
                            .ok();

                        // Send updates to reflect the last state

                        for event in Self::create_update_events(&state) {
                            let _ = event_tx.send(event);
                        }
                    } else {
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

    fn run_core_loop(
        state: &mut CoreThreadState,
        command_rx: &Receiver<Command>,
        event_tx: &Sender<Event>,
    ) {
        let mut update_counter = 0u64;

        // Loop until exited

        loop {
            // Handle pending commands

            loop {
                match command_rx.try_recv() {
                    Ok(command) => {
                        let events = command.handle(state);

                        for event in events {
                            let _ = event_tx.send(event);
                        }
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => return,
                }
            }

            // Step

            let step = matches!(state.status, CoreThreadStatus::Running)
                || matches!(state.status, CoreThreadStatus::Paused { step: true });

            if step && let Some(system) = &mut state.system {
                let mut send_update = false;

                let breakpoint_hit = system.step();

                if breakpoint_hit {
                    state.status = CoreThreadStatus::Paused { step: false };

                    let _ = event_tx.send(Event::StatusUpdate(state.status.into()));

                    send_update = true;
                } else {
                    if let CoreThreadStatus::Paused {
                        step: step_requested,
                    } = &mut state.status
                    {
                        *step_requested = false;
                        send_update = true;
                    }

                    update_counter = update_counter.wrapping_add(1);

                    if update_counter.is_multiple_of(UPDATE_INTERVAL as u64) {
                        send_update = true;
                    }
                }

                if send_update {
                    for event in Self::create_update_events(state) {
                        let _ = event_tx.send(event);
                    }
                }
            }
        }
    }

    pub fn send_command(&self, command: Command) {
        self.command_tx
            .send(command)
            .inspect_err(|error| log::error!("Failed to send command: {:?}", error))
            .ok();
    }

    pub fn poll_event(&self) -> Option<Event> {
        self.event_rx.try_recv().ok()
    }

    pub fn create_update_events(state: &CoreThreadState) -> Vec<Event> {
        let mut events = Vec::new();

        if let Some(system) = &state.system {
            if let Some(instructions_settings) = state.ui_settings.instructions.as_ref() {
                let base_address = match instructions_settings.base_address {
                    InstructionAddress::Pc => system.cpu.regs.pc,
                    //InstructionAddress::Address(addr) => addr,
                };

                let instructions = (base_address
                    ..base_address + instructions_settings.rows as u32 * 4)
                    .step_by(4)
                    .map(|addr| {
                        let instruction = system.read(addr);

                        let opcode = Opcode(instruction);
                        let handler = decode(opcode);

                        if let Some((_, disassemble)) = handler {
                            let disassembly = disassemble(system, opcode);

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
                    cpu: system.cpu,
                    cop0: system.cop0,
                    cop1: system.cop1,
                };

                events.push(Event::RegistersUpdate(registers));
            }

            if let Some(memory_settings) = state.ui_settings.memory.as_ref() {
                let base_address = memory_settings.address & !0xF;

                let data = (0..memory_settings.rows * 16)
                    .map(|i| system.try_read(base_address + i as u32))
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

            events.push(Event::AiUpdate(system.map.ai));

            events.push(Event::RspUpdate(system.map.rsp.regs));

            events.push(Event::SiUpdate(system.map.si));

            events.push(Event::IsViewerUpdate(
                system.map.cart.isviewer.get().to_string(),
            ));

            events.push(Event::CoreEventsUpdate {
                current_cycle: system.cycles,
                pending: system.pending_events(),
            });
        }

        events
    }
}
