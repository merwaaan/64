use std::collections::HashSet;

use arbitrary_int::prelude::*;
use egui::Context;
use n64_core::{
    cpu::instructions::Disassembly,
    registers::Registers,
    sp::{self, Register},
};
use strum::IntoEnumIterator;

use crate::{
    command::Command,
    event::Event,
    ui::{
        Data,
        colors::Color,
        reg32,
        text::Text,
        widgets::{ChildWidget, Widget, WidgetId},
    },
};

#[derive(Clone, Debug)]
pub struct SpUpdate {
    pub pc: u12,
    pub regs: [u32; 8],
    pub regs2: sp::Registers,
    pub instructions: Vec<(u32, Disassembly)>,
}

#[derive(Default)]
pub struct SpWidget {
    id: WidgetId,
    last_update: Option<SpUpdate>,
}

impl Widget for SpWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn requested_data(&mut self, _only_if_changed: bool) -> Option<HashSet<Data>> {
        Some(HashSet::from([Data::Sp]))
    }

    fn update(&mut self, _ctx: &Context, event: &Event) {
        if let Event::Sp(sp) = event {
            self.last_update = Some(sp.clone());
        }
    }
}

impl ChildWidget for SpWidget {
    fn show(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        if let Some(update) = &self.last_update {
            ui.columns(2, |ui| {
                ui[0].vertical(|ui| {
                    ui.horizontal(|ui| {
                        Text::new(format!("PC: {:03X}", update.pc)).show(ui);

                        // TODO
                        // if update.is_halted() {
                        //     Text::new(format!("PC: {:03X}", update.pc))
                        //         .color(Color::Warning)
                        //         .show(ui);
                        // }
                    });

                    for (address, disassembly) in &update.instructions {
                        ui.horizontal(|ui| {
                            Text::new(format!("{:03X}", address))
                                .color(Color::Active)
                                .show(ui);

                            Text::new(format!(" {}", disassembly.mnemonics)).show(ui);
                        });
                    }
                });

                ui[1].vertical(|ui| {
                    let mut show_reg = |reg: Register| {
                        reg32(ui, format!("{:>10}", reg), update.regs[reg as usize]);
                    };

                    Register::iter().for_each(|reg| {
                        show_reg(reg);
                    });

                    ui.separator();

                    for row in 0..16 {
                        ui.horizontal(|ui| {
                            for col in 0..2 {
                                let reg_index = row + col * 16;
                                let name = Registers::gpr_name(reg_index);
                                let value = update.regs2.read(reg_index);

                                reg32(ui, name, value);
                            }
                        });
                    }
                });
            });
        }

        vec![]
    }
}
