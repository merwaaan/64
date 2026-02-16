use egui::{Color32, RichText};
use n64::instructions::Disassembly;

use crate::emu::event::Event;

#[derive(Clone, Copy)]
pub enum InstructionAddress {
    Pc,
    Address(u32),
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

    instructions: Vec<InstructionData>,
    pc: u32,
}

// TODO InstructionsUpdate

impl InstructionsWidget {
    pub fn default() -> Self {
        Self {
            settings: InstructionsSettings {
                base_address: InstructionAddress::Pc,
                rows: 20,
            },
            instructions: Vec::new(),
            pc: 0,
        }
    }

    pub fn update(&mut self, event: &Event) {
        match event {
            Event::InstructionsUpdate(instructions) => {
                self.instructions = instructions.clone();
            }
            Event::RegistersUpdate(registers) => {
                self.pc = registers.cpu_regs.pc;
            }
            _ => {}
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        for instruction in &self.instructions {
            ui.horizontal(|ui| {
                ui.label({
                    let mut text = RichText::new(format!("{:08X}", instruction.address))
                        .monospace()
                        .strong();

                    if instruction.address == self.pc {
                        text = text.color(Color32::from_rgb(0, 255, 0));
                    }
                    text
                });

                ui.label(
                    RichText::new(format!(" {}", instruction.disassembly.mnemonics)).monospace(),
                );
            });
        }
    }
}
