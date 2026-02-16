use n64::breakpoints::{Breakpoint, Breakpoints};

use crate::{
    emu::{command::Command, event::Event, runner::Runner},
    ui::parse_hex,
};

#[derive(Default)]
pub struct BreakpointsWidget {
    breakpoints: Breakpoints,

    breakpoint_address: String,
}

// TODO InstructionsUpdate

impl BreakpointsWidget {
    pub fn update(&mut self, event: &Event) {
        if let Event::BreakpointsUpdate(breakpoints) = event {
            self.breakpoints = breakpoints.clone();
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, runner: &mut Option<Runner>) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.breakpoint_address);

                if ui.button("Add").clicked()
                    && let Some(runner) = runner.as_mut()
                    && let Some(address) =
                        parse_hex(&self.breakpoint_address).map(|addr| addr as u32)
                {
                    runner.send_command(Command::AddBreakpoint(Breakpoint::Address(address)));
                }
            });
        });

        for breakpoint in &self.breakpoints {
            ui.horizontal(|ui| {
                match breakpoint {
                    Breakpoint::Address(address) => {
                        ui.label(format!("{:08X}", address));
                    }
                };

                if ui.button("Remove").clicked()
                    && let Some(runner) = runner.as_mut()
                {
                    runner.send_command(Command::RemoveBreakpoint(*breakpoint));
                };
            });
        }
    }
}
