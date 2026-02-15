use std::path::PathBuf;

use crate::{
    emu::{command::Command, runner::Runner},
    ui::{instructions::InstructionsWidget, memory::MemoryWidget, registers::RegistersWidget},
};

pub mod instructions;
pub mod memory;
pub mod registers;

#[derive(Clone)]
pub struct InstructionsSettings {
    pub address: u32,
    pub rows: usize,
}

#[derive(Clone)]
pub struct MemorySettings {
    pub address: u32,
    pub rows: usize, // 16 bytes per row
}

#[derive(Clone)]
pub struct UiSettings {
    pub instructions: Option<InstructionsSettings>,
    pub cpu_regs: Option<bool>,
    pub memory: Option<MemorySettings>,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            instructions: Some(InstructionsSettings {
                address: 0,
                rows: 8,
            }),
            cpu_regs: Some(true),
            memory: Some(MemorySettings {
                address: 0,
                rows: 4,
            }),
        }
    }
}

pub struct Ui {
    runner: Option<Runner>,
    instructions: Option<InstructionsWidget>,
    registers: Option<RegistersWidget>,
    memory: Option<MemoryWidget>,
    paused: bool,
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
            instructions: Some(InstructionsWidget::default()),
            registers: Some(RegistersWidget::default()),
            memory: Some(MemoryWidget::default()),
            paused: true,
        }
    }

    fn poll_runner(&mut self) {
        while let Some(event) = self.runner.as_ref().and_then(|r| r.poll_event()) {
            if let Some(instructions) = self.instructions.as_mut() {
                instructions.update(&event);
            }
            if let Some(registers) = self.registers.as_mut() {
                registers.update(&event);
            }
            if let Some(memory) = self.memory.as_mut() {
                memory.update(&event);
            }
        }
    }
}

impl eframe::App for Ui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_runner();

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Load ROM…").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("ROM files", &["n64", "z64", "v64"])
                            .pick_file()
                        {
                            if let Some(runner) = &self.runner {
                                runner.send_command(Command::LoadRom { path });
                            }
                        }

                        ui.close_kind(egui::UiKind::Menu);
                    }
                });

                if ui
                    .button(if self.paused {
                        "▶ Resume"
                    } else {
                        "⏸ Pause"
                    })
                    .clicked()
                {
                    if let Some(runner) = &self.runner {
                        if self.paused {
                            runner.send_command(Command::Resume);
                        } else {
                            runner.send_command(Command::Pause);
                        }
                        self.paused = !self.paused;
                    }
                }

                if self.paused {
                    if ui.button("⏭ Step").clicked() {
                        if let Some(runner) = &self.runner {
                            runner.send_command(Command::Step);
                        }
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical(|ui| {
                    if let Some(instructions) = self.instructions.as_ref() {
                        instructions.show(ui);
                    }

                    if let Some(registers) = self.registers.as_ref() {
                        registers.show(ui);
                    }

                    if let Some(memory) = self.memory.as_ref() {
                        memory.show(ui);
                    }
                });
            });
        });

        ctx.request_repaint();
    }
}
