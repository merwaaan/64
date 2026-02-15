use n64::instructions::Disassembly;

use crate::emu::event::Event;

#[derive(Clone)]
pub struct InstructionData {
    pub address: u32,
    pub disassembly: Disassembly,
}

#[derive(Default)]
pub struct InstructionsWidget {
    pub instructions: Vec<InstructionData>,
}

impl InstructionsWidget {
    pub fn update(&mut self, event: &Event) {
        if let Event::Update {
            instructions: Some(instructions),
            ..
        } = event
        {
            self.instructions = instructions.clone();
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Instructions")
            .default_open(true)
            .show(ui, |ui| {
                for instruction in &self.instructions {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{:08X}", instruction.address))
                                .monospace()
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new(format!(" {}", instruction.disassembly.mnemonics))
                                .monospace(),
                        );
                    });
                }
            });
    }
}
