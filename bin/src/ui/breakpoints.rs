use egui::{Context, Grid, Window};
use n64::breakpoints::Breakpoints;

use crate::{
    emu::{command::Command, event::Event},
    ui::{Widget, colors::Color, parse_hex, text::Text},
};

#[derive(Default)]
pub struct BreakpointsWidget {
    breakpoints: Breakpoints,
    pc: u32,

    breakpoint_address: String,
}

// TODO InstructionsUpdate

impl Widget for BreakpointsWidget {
    fn update(&mut self, _ctx: &Context, event: &Event) {
        match event {
            Event::BreakpointsUpdate(breakpoints) => {
                self.breakpoints = breakpoints.clone();
            }
            Event::RegistersUpdate(registers) => {
                self.pc = registers.cpu_regs.pc;
            }
            _ => {}
        }
    }

    fn show(&mut self, ctx: &Context) -> Vec<Command> {
        let mut commands = Vec::new();

        Window::new("Breakpoints")
            .default_pos([900.0, 1400.0])
            .show(ctx, |ui| {
                Grid::new("breakpoints").show(ui, |ui| {
                    // Input

                    ui.text_edit_singleline(&mut self.breakpoint_address);

                    if ui.button("Add").clicked()
                        && let Some(address) =
                            parse_hex(&self.breakpoint_address).map(|addr| addr as u32)
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

        commands
    }
}
