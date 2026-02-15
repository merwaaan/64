use n64::registers::Registers;

use crate::emu::event::Event;

#[derive(Default)]
pub struct RegistersWidget {
    pub cpu_regs: Registers,
}

impl RegistersWidget {
    pub fn update(&mut self, event: &Event) {
        if let Event::Update {
            cpu_regs: Some(cpu_regs),
            ..
        } = event
        {
            self.cpu_regs = cpu_regs.clone();
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Registers")
            .default_open(true)
            .show(ui, |ui| {
                ui.monospace(format!("PC: {:08X}", self.cpu_regs.pc));
                ui.add_space(4.0);

                ui.horizontal_top(|ui| {
                    for col in 0..2 {
                        ui.vertical(|ui| {
                            for row in 0..16 {
                                let reg_index = col * 16 + row;
                                let name = Registers::gpr_name(reg_index);
                                let value = self.cpu_regs.gpr[reg_index].get64();
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(name).color(egui::Color32::from_rgb_additive(200, 200, 255)).strong(),
                                    );
                                    ui.monospace(format!("{:08X} {:08X}", (value >> 32) as u32, value as u32));
                                });
                            }
                        });
                        ui.add_space(20.0);
                    }
                });
            });
    }
}
