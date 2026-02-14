use std::{path::PathBuf, time::Duration};

use crossbeam::channel::TryRecvError;
use iced::{Element, Subscription, Theme, time, widget::column};

use crate::{
    emu::{command::Command, runner::Runner},
    ui::{instructions::InstructionsWidget, memory::MemoryWidget, registers::RegistersWidget},
};

pub mod instructions;
pub mod memory;
pub mod registers;
mod theme;

pub enum UiEvent {
    Refresh,
}

//pub struct UiState {}

pub struct Ui {
    runner: Option<Runner>,
    //state: UiState,
    instructions: InstructionsWidget,
    registers: RegistersWidget,
    memory: MemoryWidget,
}

impl Ui {
    pub fn new() -> Self {
        let runner = Runner::new();

        // TODO temp
        runner.send_command(Command::LoadRom {
            path: PathBuf::from("roms/sm.n64"),
        });

        Self {
            runner: Some(runner),
            //state: UiState {},
            instructions: InstructionsWidget::default(),
            registers: RegistersWidget::default(),
            memory: MemoryWidget::default(),
        }
    }

    pub fn theme(&self) -> Theme {
        Theme::CatppuccinMacchiato
    }

    pub fn subscribe(&self) -> Subscription<UiEvent> {
        time::every(Duration::from_millis(100)).map(|_| UiEvent::Refresh)
    }

    pub fn update(&mut self, event: UiEvent) {
        // TODO poll the runner "manually" for now, iced patterns are a mess

        if let Some(runner) = &mut self.runner {
            loop {
                match runner.event_rx.try_recv() {
                    Ok(event) => {
                        self.instructions.update(&event);
                        self.registers.update(&event);
                        self.memory.update(&event);
                    }
                    Err(TryRecvError::Empty) => {
                        break;
                    }
                    Err(error) => {
                        log::error!("Runner channel error: {:?}", error);
                        self.runner = None;
                        break;
                    }
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, UiEvent> {
        column![
            self.instructions.view(),
            self.registers.view(),
            self.memory.view()
        ]
        .into()
    }
}
