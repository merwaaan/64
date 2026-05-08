use std::simd::*;
use std::{collections::HashSet, simd::num::SimdInt};

use arbitrary_int::prelude::*;
use egui::Context;
use n64_core::registers::Registers;
use n64_core::sp;

use crate::{
    command::Command,
    event::Event,
    ui::{
        Data, colors, reg32,
        text::Text,
        widgets::{ChildWidget, Widget, WidgetId},
    },
};

#[derive(Clone, Debug)]
pub struct SpUpdate {
    pub pc: u12,
    pub regs: n64_specs::rsp::Registers,
    pub regs2: sp::Registers,
    pub vregs: [i16x8; 32],
    pub vacc: i64x8,
    pub vco: u16,
    pub vcc: u16,
    pub vce: u8,
    pub instructions: Vec<(u32, String)>,
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
                                .color(colors::ACTIVE)
                                .show(ui);

                            Text::new(disassembly).show(ui);
                        });
                    }
                });

                ui[1].vertical(|ui| {
                    // TODO fix

                    // let mut show_reg = |reg: n64_specs::rsp::Register| {
                    //     reg32(ui, format!("{:>10}", reg), reg.raw_value());
                    // };

                    // n64_specs::rsp::Register::iter().for_each(|reg| {
                    //     show_reg(reg);
                    // });

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

                    ui.separator();

                    // Accumulator

                    ui.horizontal(|ui| {
                        Text::new("ACC HI").color(colors::LIGHT).show(ui);

                        let data = (update.vacc >> 32).cast::<i16>();

                        for i in 0..8 {
                            Text::new(format!("{:04X}", data[i])).show(ui);
                        }
                    });

                    ui.horizontal(|ui| {
                        Text::new("ACC MI").color(colors::LIGHT).show(ui);

                        let data = (update.vacc >> 16).cast::<i16>();

                        for i in 0..8 {
                            Text::new(format!("{:04X}", data[i])).show(ui);
                        }
                    });

                    ui.horizontal(|ui| {
                        Text::new("ACC LO").color(colors::LIGHT).show(ui);

                        let data = update.vacc.cast::<i16>();

                        for i in 0..8 {
                            Text::new(format!("{:04X}", data[i])).show(ui);
                        }
                    });

                    Text::new(format!("VCO: {:04X}", update.vco)).show(ui);
                    Text::new(format!("VCC: {:04X}", update.vcc)).show(ui);
                    Text::new(format!("VCE: {:02X}", update.vce)).show(ui);

                    for row in 0..32 {
                        ui.horizontal(|ui| {
                            Text::new(format!("V{:02}", row))
                                .color(colors::LIGHT)
                                .show(ui);

                            let data = update.vregs[row].to_array();

                            for i in &data {
                                Text::new(format!("{:04X}", i)).show(ui);
                            }
                        });
                    }
                });
            });
        }

        vec![]
    }
}
