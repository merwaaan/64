use n64_core::{breakpoints::Breakpoints, instructions::Disassembly};
use egui::{Context, CursorIcon, Grid, Window};

use crate::{
    emu::{command::Command, event::Event},
    ui::{SettingUpdate, Widget, colors::Color, memory::MemorySettings, parse_hex, text::Text},
};

#[derive(Clone, Copy)]
pub enum InstructionAddress {
    Pc,
    //Address(u32),
}

#[derive(Clone, Copy)]
pub struct InstructionsSettings {
    pub base_address: InstructionAddress,
    pub rows: usize,
}

#[derive(Clone)]
pub struct InstructionData {
    pub address: u32,
    pub disassembly: Disassembly,
}

pub struct InstructionsWidget {
    pub settings: InstructionsSettings,

    pc: u32,
    instructions: Vec<InstructionData>,

    breakpoints: Breakpoints,
    breakpoint_input_address: String,
}

impl Default for InstructionsWidget {
    fn default() -> Self {
        Self {
            settings: InstructionsSettings {
                base_address: InstructionAddress::Pc,
                rows: 20,
            },

            pc: 0,
            instructions: Vec::new(),

            breakpoints: Breakpoints::default(),
            breakpoint_input_address: String::new(),
        }
    }
}

impl Widget for InstructionsWidget {
    fn init(&mut self) -> Vec<Command> {
        vec![Command::SetSetting(SettingUpdate::Instructions(Some(
            self.settings,
        )))]
    }

    fn update(&mut self, _ctx: &Context, event: &Event) {
        match event {
            Event::InstructionsUpdate(instructions) => {
                self.instructions = instructions.clone();
            }
            Event::RegistersUpdate(registers) => {
                self.pc = registers.cpu.regs.pc;
            }
            Event::BreakpointsUpdate(breakpoints) => {
                self.breakpoints = breakpoints.clone();
            }
            _ => {}
        }
    }

    fn show(&mut self, ctx: &Context) -> Vec<Command> {
        let mut commands = Vec::new();

        Window::new("Instructions")
            .default_pos([0.0, 100.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        for instruction in &self.instructions {
                            ui.horizontal(|ui| {
                                let maybe_breakpoint = self.breakpoints.get(instruction.address);

                                match maybe_breakpoint {
                                    Some(breakpoint) => {
                                        Text::new("•")
                                            .color(if breakpoint.enabled() {
                                                Color::Active
                                            } else {
                                                Color::Light
                                            })
                                            .show(ui);
                                    }
                                    None => {
                                        Text::new(" ").show(ui);
                                    }
                                }

                                let opcode_response =
                                    Text::new(format!("{:08X}", instruction.address))
                                        .color(Color::Active)
                                        .reverse(instruction.address == self.pc)
                                        .show(ui);

                                if opcode_response.hovered() {
                                    ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                                }

                                opcode_response.context_menu(|ui| {
                                    ui.set_min_width(200.0);

                                    if let Some(breakpoint) =
                                        self.breakpoints.get(instruction.address)
                                    {
                                        let label = if breakpoint.enabled() {
                                            "Disable breakpoint"
                                        } else {
                                            "Enable breakpoint"
                                        };

                                        if ui.button(label).clicked() {
                                            commands.push(Command::ToggleBreakpoint(
                                                instruction.address,
                                            ));
                                        }

                                        if ui.button("Delete breakpoint").clicked() {
                                            commands.push(Command::RemoveBreakpoint(
                                                instruction.address,
                                            ));
                                        }
                                    } else if ui.button("Add breakpoint").clicked() {
                                        commands.push(Command::AddBreakpoint(instruction.address));
                                    }

                                    if ui.button("Show in memory").clicked() {
                                        commands.push(Command::SetSetting(SettingUpdate::Memory(
                                            Some(MemorySettings {
                                                address: instruction.address,
                                                rows: 8, // TODO hack, should be moved up
                                            }),
                                        )));
                                    }
                                });

                                Text::new(format!(" {}", instruction.disassembly.mnemonics))
                                    .show(ui);
                            });
                        }
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        Grid::new("breakpoints").show(ui, |ui| {
                            // Input

                            ui.text_edit_singleline(&mut self.breakpoint_input_address);

                            if ui.button("Add").clicked()
                                && let Some(address) = parse_hex(&self.breakpoint_input_address)
                                    .map(|addr| addr as u32)
                            {
                                commands.push(Command::AddBreakpoint(address));
                            }

                            ui.end_row();

                            // Breakpoints

                            for (address, enabled) in self.breakpoints.iter() {
                                let mut enabled_value = enabled;
                                if ui.checkbox(&mut enabled_value, "").changed() {
                                    commands.push(Command::ToggleBreakpoint(address));
                                }

                                Text::new(format!("{:08X}", address))
                                    .bold()
                                    .color(if address == self.pc {
                                        Color::Active
                                    } else {
                                        Color::Default
                                    })
                                    .show(ui);

                                if ui.button("Remove").clicked() {
                                    commands.push(Command::RemoveBreakpoint(address));
                                };

                                ui.end_row();
                            }
                        });
                    });
                })
            });

        commands
    }
}
