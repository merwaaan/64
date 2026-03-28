use std::collections::{HashMap, HashSet};
use std::panic::{self, AssertUnwindSafe};
use std::thread::{self, JoinHandle};

use crossbeam::channel::{Receiver, Sender, TryRecvError, unbounded};
use n64_core::{
    cpu,
    cpu::instructions::Disassembly,
    cpu::opcode::Opcode,
    sp,
    system::{Address, System},
    value::Value,
    vi::Vi,
};

use crate::ui::widgets::ai_widget::AiUpdate;
use crate::ui::widgets::memory_widget::MemoryUpdate;
use crate::{
    command::Command,
    event::Event,
    ui::{
        Data, Status,
        widgets::{
            WidgetId, cpu_widget::CpuUpdate, framebuffer_widget::FramebufferUpdate,
            sp_widget::SpUpdate,
        },
    },
};

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
}

pub struct CoreThread {
    _thread: JoinHandle<()>,
    command_tx: Sender<Command>,
    event_rx: Receiver<Event>,
}

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

                // We keep the requested data out of the state to avoid losing it on panics
                let mut requested_data = HashMap::<WidgetId, HashSet<Data>>::new();

                loop {
                    // Run the core loop

                    let mut state = CoreThreadState {
                        system: None,
                        status: CoreThreadStatus::Paused { step: false },
                    };

                    let result = panic::catch_unwind(AssertUnwindSafe(|| {
                        Self::run_core_loop(
                            &mut state,
                            &mut requested_data,
                            &command_rx,
                            &event_tx,
                        );
                    }));

                    // The core loop exites, two possible reasons:
                    // - it exited cleanly: just exit the thread
                    // - it panicked: restart the core loop

                    if result.is_err() {
                        log::error!("Core loop panicked");

                        // This might panic again as we're reading a possibly invalid state

                        panic::catch_unwind(AssertUnwindSafe(|| {
                            // Notify the UI

                            event_tx
                                .send(Event::Status(Status::Panicked(format!(
                                    "{:?}",
                                    result.unwrap_err()
                                ))))
                                .inspect_err(|error| {
                                    log::error!("Failed to send status update: {:?}", error)
                                })
                                .ok();

                            // Send updates to reflect the last state

                            for event in Self::create_update_events(&mut state, &requested_data) {
                                let _ = event_tx.send(event);
                            }
                        }))
                        .ok();
                    } else {
                        break;
                    }
                }
            })
            .expect("Failed to spawn core thread");

        Self {
            _thread: thread,
            command_tx,
            event_rx,
        }
    }

    fn run_core_loop(
        state: &mut CoreThreadState,
        requested_data: &mut HashMap<WidgetId, HashSet<Data>>,
        command_rx: &Receiver<Command>,
        event_tx: &Sender<Event>,
    ) {
        // TODO clean this up, once every n frames?
        const RECEIVE_COMMANDS_INTERVAL: usize = 1_000;
        const SEND_EVENTS_INTERVAL: usize = 1_000_000;

        let mut update_counter: usize = 0;

        // Loop until exited

        loop {
            // Handle pending commands

            if update_counter.is_multiple_of(RECEIVE_COMMANDS_INTERVAL) {
                loop {
                    match command_rx.try_recv() {
                        Ok(command) => {
                            let events = command.handle(state, requested_data);

                            for event in events {
                                let _ = event_tx.send(event);
                            }
                        }
                        Err(TryRecvError::Empty) => break,
                        Err(TryRecvError::Disconnected) => return,
                    }
                }
            }

            let mut send_events = false;

            let step = matches!(state.status, CoreThreadStatus::Running)
                || matches!(state.status, CoreThreadStatus::Paused { step: true });

            if step && let Some(system) = &mut state.system {
                let breakpoint_hit = system.step();

                if breakpoint_hit {
                    state.status = CoreThreadStatus::Paused { step: false };

                    let _ = event_tx.send(Event::Status(state.status.into()));

                    send_events = true;
                } else {
                    if let CoreThreadStatus::Paused {
                        step: step_requested,
                    } = &mut state.status
                    {
                        *step_requested = false;
                        send_events = true;
                    }
                }
            }

            if update_counter.is_multiple_of(SEND_EVENTS_INTERVAL) {
                send_events = true;
            }

            if send_events {
                for event in Self::create_update_events(state, requested_data) {
                    let _ = event_tx.send(event);
                }
            }

            update_counter = update_counter.wrapping_add(1);
        }
    }

    pub fn send_command(&self, command: Command) {
        self.command_tx
            .send(command)
            //.inspect_err(|error| log::error!("Failed to send command: {:?}", error))
            .ok();
    }

    pub fn poll_event(&self) -> Option<Event> {
        self.event_rx.try_recv().ok()
    }

    pub fn create_update_events(
        state: &mut CoreThreadState,
        requested_data: &HashMap<WidgetId, HashSet<Data>>,
    ) -> Vec<Event> {
        let mut events = Vec::new();

        if let Some(system) = &mut state.system {
            for (_id, data) in requested_data {
                for data in data {
                    match data {
                        Data::Memory(settings) => {
                            let base_address = settings.address & !0xF;

                            let data = (0..settings.rows * 16)
                                .map(|i| system.peek(Address::p(base_address + i as u32)))
                                .collect();

                            events.push(Event::Memory(MemoryUpdate { base_address, data }));
                        }

                        Data::Cpu => {
                            let base_address = system.cpu.regs.pc;

                            let instructions = (base_address..base_address + 20 * 4)
                                .step_by(4)
                                .map(|addr| {
                                    system
                                        .peek(Address::v(addr))
                                        .map(|instruction| {
                                            let opcode = Opcode(instruction);

                                            let (_execute, disassemble) =
                                                cpu::instructions::decode(opcode);

                                            (addr, disassemble(system, opcode))
                                        })
                                        .unwrap_or((
                                            addr,
                                            Disassembly::new("<CANNOT DECODE>".to_string()),
                                        ))
                                })
                                .collect();

                            events.push(Event::Cpu(CpuUpdate {
                                cpu: system.cpu,
                                instructions,
                            }));
                        }

                        Data::Cop0 => {
                            events.push(Event::Cop0(system.cop0));
                        }

                        Data::Cop1 => {
                            events.push(Event::Cop1(system.cop1));
                        }

                        Data::Sp => {
                            let base_imem_addr = u32::from(system.sp.pc);

                            let instructions = (0..20)
                                .map(|offset| {
                                    let addr = (base_imem_addr + offset * 4) & 0x0FFF;

                                    let instruction = u32::read_mem(&system.sp.mem, 0x1000 + addr);
                                    let opcode = Opcode(instruction);
                                    let handler = sp::instructions::decode(opcode);

                                    if let Some((_, disassemble)) = handler {
                                        (addr, disassemble(system, opcode))
                                    } else {
                                        (
                                            addr,
                                            Disassembly::new(format!(
                                                "<UNKNOWN {:08X}>",
                                                instruction
                                            )),
                                        )
                                    }
                                })
                                .collect();

                            events.push(Event::Sp(SpUpdate {
                                pc: system.sp.pc,
                                instructions,
                                regs: system.sp.regs,
                                regs2: system.sp.regs2,
                                vregs: system.sp.vregs,
                                vacc: system.sp.vacc,
                                vco: system.sp.vco,
                                vcc: system.sp.vcc,
                                vce: system.sp.vce,
                            }));
                        }

                        Data::Mi => {
                            events.push(Event::Mi(system.mi));
                        }

                        Data::Vi => {
                            events.push(Event::Vi(system.vi));
                        }

                        Data::Ai => {
                            events.push(Event::Ai(AiUpdate {
                                ai: system.ai,
                                queued_samples: system.audio_renderer.queued_samples(),
                            }));
                        }

                        Data::Pi => {
                            events.push(Event::Pi(system.pi));
                        }

                        Data::Dp => {
                            events.push(Event::Dp(system.dp.regs));
                        }

                        Data::Si => {
                            events.push(Event::Si(system.si));
                        }

                        Data::Tlb => {
                            events.push(Event::Tlb(system.cop0.tlb));
                        }

                        Data::IsViewer => {
                            events.push(Event::IsViewer(system.cart.isviewer.get().to_string()));
                        }

                        Data::Breakpoints => {
                            events.push(Event::Breakpoints(system.breakpoints().clone()));
                        }

                        Data::Events => {
                            events.push(Event::Events {
                                current_cycle: system.cpu.cycles(),
                                pending: system.pending_events(),
                            });
                        }

                        Data::Framebuffer => {
                            //let (data, width, height) = Vi::extract_framebuffer(system);

                            let (data, width, height) = system.video_renderer.get_frame();

                            // if data.len() > 0 {
                            //     log::debug!(
                            //         "Framebuffer data: {:?} {:?}",
                            //         data.len(),
                            //         &data[..100]
                            //     );
                            // }
                            events.push(Event::Framebuffer(FramebufferUpdate {
                                width,
                                height,
                                data,
                            }));
                        }
                    }
                }
            }
        }

        events
    }
}
