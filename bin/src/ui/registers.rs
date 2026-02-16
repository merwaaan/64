use egui::{Color32, RichText};
use n64::{
    cop0::Cop0,
    registers::{Reg64, Registers},
};

use crate::emu::event::Event;

#[derive(Default)]
pub struct RegistersWidget {
    last_update: Option<RegistersUpdate>,
}

#[derive(Clone, Copy)]
pub struct RegistersUpdate {
    pub cpu_regs: Registers,
    pub cop0_regs: [Reg64; 32],
}

impl RegistersWidget {
    pub fn update(&mut self, event: &Event) {
        if let Event::RegistersUpdate(update) = event {
            self.last_update = Some(*update);
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        if let Some(last_update) = &self.last_update {
            ui.monospace(format!("PC: {:08X}", last_update.cpu_regs.pc));

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("HI:");
                ui.monospace(format!(
                    "{:08X} {:08X}",
                    (last_update.cpu_regs.mult_hi.get64() >> 32) as u32,
                    last_update.cpu_regs.mult_hi.get64() as u32
                ));

                ui.add_space(4.0);

                ui.label("LO:");
                ui.monospace(format!(
                    "{:08X} {:08X}",
                    (last_update.cpu_regs.mult_lo.get64() >> 32) as u32,
                    last_update.cpu_regs.mult_lo.get64() as u32
                ));
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                for col in 0..2 {
                    ui.vertical(|ui| {
                        for row in 0..16 {
                            let reg_index = col * 16 + row;
                            let name = Registers::gpr_name(reg_index);
                            let value = last_update.cpu_regs.gpr[reg_index].get64();
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(name)
                                        .color(Color32::from_rgb_additive(200, 200, 255))
                                        .strong(),
                                );
                                ui.monospace(format!(
                                    "{:08X} {:08X}",
                                    (value >> 32) as u32,
                                    value as u32
                                ));
                            });
                        }
                    });

                    ui.add_space(20.0);
                }
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                for col in 0..2 {
                    ui.vertical(|ui| {
                        for row in 0..16 {
                            let reg_index = col * 16 + row;
                            let value = last_update.cop0_regs[reg_index].get64();

                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("{:>8}", Cop0::reg_name(reg_index)))
                                        .monospace()
                                        .color(Color32::from_rgb_additive(200, 200, 255))
                                        .strong(),
                                );
                                ui.monospace(format!(
                                    "{:08X} {:08X}",
                                    (value >> 32) as u32,
                                    value as u32
                                ));
                            });
                        }
                    });

                    ui.add_space(20.0);
                }
            });
        }
    }
}
