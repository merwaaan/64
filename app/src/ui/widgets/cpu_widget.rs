use std::collections::HashSet;

use egui::CursorIcon;
use n64_core::{breakpoints::Breakpoints, cpu::Cpu, registers::Registers};

use crate::{
    command::Command,
    event::Event,
    ui::{
        Data, colors, reg32, reg64,
        text::Text,
        widgets::{ChildWidget, Widget, WidgetId},
    },
};

#[derive(Clone, Debug)]
pub struct CpuUpdate {
    pub cpu: Cpu,
    pub instructions: Vec<(u32, String)>,
}

#[derive(Default)]
pub struct CpuWidget {
    id: WidgetId,
    last_update: Option<CpuUpdate>,

    breakpoints: Breakpoints,
    breakpoint_input_address: String,
}

impl Widget for CpuWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn requested_data(&mut self, _only_if_changed: bool) -> Option<HashSet<Data>> {
        Some(HashSet::from([Data::Cpu, Data::Breakpoints]))
    }

    fn update(&mut self, _context: &egui::Context, event: &Event) {
        match event {
            Event::Cpu(cpu) => {
                self.last_update = Some(cpu.clone());
            }
            Event::Breakpoints(breakpoints) => {
                self.breakpoints = breakpoints.clone();
            }
            _ => {}
        }
    }
}

impl ChildWidget for CpuWidget {
    fn show(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        let commands = Vec::new();

        if let Some(last_update) = &self.last_update {
            ui.columns(2, |ui| {
                ui[0].vertical(|ui| {
                    for (address, disassembly) in &last_update.instructions {
                        ui.horizontal(|ui| {
                            let maybe_breakpoint = self.breakpoints.get(*address);

                            match maybe_breakpoint {
                                Some(breakpoint) => {
                                    Text::new("•")
                                        .color(if breakpoint.enabled() {
                                            colors::ACTIVE
                                        } else {
                                            colors::LIGHT
                                        })
                                        .show(ui);
                                }
                                None => {
                                    Text::new(" ").show(ui);
                                }
                            }

                            let opcode_response = Text::new(format!("{:08X}", address))
                                .color(colors::ACTIVE)
                                .reverse(*address == last_update.cpu.regs.pc)
                                .show(ui);

                            if opcode_response.hovered() {
                                ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                            }

                            // opcode_response.context_menu(|ui| {
                            //     ui.set_min_width(200.0);

                            //     if let Some(breakpoint) = self.breakpoints.get(instruction.address)
                            //     {
                            //         let label = if breakpoint.enabled() {
                            //             "Disable breakpoint"
                            //         } else {
                            //             "Enable breakpoint"
                            //         };

                            //         if ui.button(label).clicked() {
                            //             commands
                            //                 .push(Command::ToggleBreakpoint(instruction.address));
                            //         }

                            //         if ui.button("Delete breakpoint").clicked() {
                            //             commands
                            //                 .push(Command::RemoveBreakpoint(instruction.address));
                            //         }
                            //     } else if ui.button("Add breakpoint").clicked() {
                            //         commands.push(Command::AddBreakpoint(instruction.address));
                            //     }

                            //     if ui.button("Show in memory").clicked() {
                            //         commands.push(Command::SetSetting(SettingUpdate::Memory(
                            //             Some(MemorySettings {
                            //                 address: instruction.address,
                            //                 rows: 8, // TODO hack, should be moved up
                            //             }),
                            //         )));
                            //     }
                            // });

                            Text::new(disassembly).show(ui);
                        });
                    }
                });

                ui[1].vertical(|ui| {
                    reg64(ui, "CYCLES", last_update.cpu.cycles() as u64);

                    reg64(ui, "PC", last_update.cpu.regs.pc as u64);

                    ui.horizontal(|ui| {
                        reg64(ui, "HI", last_update.cpu.regs.mult_hi.get64());
                        reg64(ui, "LO", last_update.cpu.regs.mult_lo.get64());
                    });

                    reg32(ui, "LLBit", last_update.cpu.regs.load_linked_bit as u32);

                    // GPR

                    for row in 0..16 {
                        ui.horizontal(|ui| {
                            for col in 0..2 {
                                let reg_index = row + col * 16;
                                let name = Registers::gpr_name(reg_index);
                                let value = last_update.cpu.regs.gpr[reg_index].get64();

                                reg64(ui, name, value);
                            }
                        });
                    }
                });
            });

            // ui.separator();

            // ui.vertical(|ui| {
            //     Grid::new("breakpoints").show(ui, |ui| {
            //         // Input

            //         ui.text_edit_singleline(&mut self.breakpoint_input_address);

            //         if ui.button("Add").clicked()
            //             && let Some(address) =
            //                 parse_hex(&self.breakpoint_input_address).map(|addr| addr as u32)
            //         {
            //             commands.push(Command::AddBreakpoint(address));
            //         }

            //         ui.end_row();

            //         // Breakpoints

            //         for (address, enabled) in self.breakpoints.iter() {
            //             let mut enabled_value = enabled;
            //             if ui.checkbox(&mut enabled_value, "").changed() {
            //                 commands.push(Command::ToggleBreakpoint(address));
            //             }

            //             Text::new(format!("{:08X}", address))
            //                 .bold()
            //                 .color(if address == last_update.cpu.regs.pc {
            //                     Color::Active
            //                 } else {
            //                     Color::Default
            //                 })
            //                 .show(ui);

            //             if ui.button("Remove").clicked() {
            //                 commands.push(Command::RemoveBreakpoint(address));
            //             };

            //             ui.end_row();
            //         }
            //     });
            // });
        }

        commands
    }
}
