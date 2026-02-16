use egui::{Color32, RichText};
use n64::{
    cop0::Cop0,
    mi::{Interrupt, Mi},
    registers::{Reg64, Registers},
};

use crate::emu::event::Event;

#[derive(Default)]
pub struct MiWidget {
    last_update: Option<Mi>,
}

impl MiWidget {
    pub fn update(&mut self, event: &Event) {
        if let Event::MiUpdate(mi) = event {
            self.last_update = Some(*mi);
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        if let Some(mi) = &self.last_update {
            let mut reg = |reg: usize| {
                ui.horizontal(|ui| {
                    ui.monospace(format!("{:>11}", Mi::reg_name(reg)));
                    ui.monospace(format!("{:08X}", mi.regs[reg]));
                });
            };

            reg(0);
            reg(1);
            reg(2);
            reg(3);

            // ui.label("Upper mode");
            // ui.label(format!("{}", mi.upper_mode()));
            // ui.label("EBus mode");
            // ui.label(format!("{}", mi.ebus_mode()));
            // ui.label("Repeat mode");
            // ui.label(format!("{}", mi.repeat_mode()));
            // ui.label("Repeat count");
            // ui.label(format!("{}", mi.repeat_count()));

            ui.separator();

            ui.horizontal(|ui| {
                let mut int = |interrupt: Interrupt| {
                    ui.monospace(RichText::new(format!("{:?}", interrupt)).color(
                        if mi.has_pending_interrupt(interrupt) {
                            if mi.is_interrupt_masked(interrupt) {
                                Color32::YELLOW
                            } else {
                                Color32::GREEN
                            }
                        } else {
                            Color32::RED
                        },
                    ));
                };

                int(Interrupt::Sp);
                int(Interrupt::Si);
                int(Interrupt::Ai);
                int(Interrupt::Vi);
                int(Interrupt::Pi);
                int(Interrupt::Dp);
            });
        }
    }
}
