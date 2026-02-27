use egui::{Context, Window};
use n64::instructions::Disassembly;

use crate::{
    emu::{command::Command, event::Event},
    ui::{SettingUpdate, Widget, colors::Color, text::Text},
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

    instructions: Vec<InstructionData>,
    pc: u32,
}

impl Default for InstructionsWidget {
    fn default() -> Self {
        Self {
            settings: InstructionsSettings {
                base_address: InstructionAddress::Pc,
                rows: 20,
            },
            instructions: Vec::new(),
            pc: 0,
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
            _ => {}
        }
    }

    fn show(&mut self, ctx: &Context) -> Vec<Command> {
        Window::new("Instructions")
            .default_pos([0.0, 100.0])
            .show(ctx, |ui| {
                for instruction in &self.instructions {
                    ui.horizontal(|ui| {
                        Text::new("•").color(Color::Light).show(ui);

                        Text::new(format!("{:08X}", instruction.address))
                            .color(Color::Active)
                            .reverse(instruction.address == self.pc)
                            .show(ui);

                        Text::new(format!(" {}", instruction.disassembly.mnemonics)).show(ui);
                    });
                }
            });

        vec![]
    }
}
