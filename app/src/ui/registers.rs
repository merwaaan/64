use egui::{Context, Window};
use n64_core::{cop0::Cop0, cop1::Cop1, cpu::Cpu, registers::Registers};

use crate::{
    emu::{command::Command, event::Event},
    ui::{SettingUpdate, Widget, reg32, reg64},
};

#[derive(Default)]
pub struct RegistersWidget {
    last_update: Option<RegistersUpdate>,
}

#[derive(Clone, Copy)]
pub struct RegistersUpdate {
    pub cpu: Cpu,
    pub cop0: Cop0,
    pub cop1: Cop1,
}

impl Widget for RegistersWidget {
    fn init(&mut self) -> Vec<Command> {
        vec![Command::SetSetting(SettingUpdate::Registers(Some(())))]
    }

    fn update(&mut self, _ctx: &Context, event: &Event) {
        if let Event::RegistersUpdate(update) = event {
            self.last_update = Some(*update);
        }
    }

    fn show(&mut self, ctx: &Context) -> Vec<Command> {
        Window::new("Registers")
            .default_pos([800.0, 100.0])
            .show(ctx, |ui| {
                if let Some(last_update) = &self.last_update {
                    reg64(ui, "STEP", last_update.cpu.step as u64);

                    reg64(ui, "PC", last_update.cpu.regs.pc as u64);

                    ui.horizontal(|ui| {
                        reg64(ui, "HI", last_update.cpu.regs.mult_hi.get64());
                        reg64(ui, "LO", last_update.cpu.regs.mult_lo.get64());
                    });

                    reg32(ui, "LLBit", last_update.cpu.regs.load_linked_bit as u32);

                    // GPR

                    for row in 0..16 {
                        ui.horizontal(|ui| {
                            for col in 0..2 {
                                let reg_index = row + col * 16;
                                let name = Registers::gpr_name(reg_index);
                                let value = last_update.cpu.regs.gpr[reg_index].get64();

                                reg64(ui, name, value);
                            }
                        });
                    }

                    ui.separator();

                    // COP0

                    for row in 0..16 {
                        ui.horizontal(|ui| {
                            for col in 0..2 {
                                let reg_index = row + col * 16;
                                let name = format!("{:>8}", Cop0::reg_name(reg_index));
                                let value = last_update.cop0.read(reg_index).get64();

                                reg64(ui, name, value);
                            }
                        });
                    }

                    ui.separator();

                    // COP1

                    for row in 0..16 {
                        ui.horizontal(|ui| {
                            for col in 0..2 {
                                let reg_index = row + col * 16;
                                let name = format!("{:>3}", Registers::fpr_name(reg_index));
                                let value = last_update.cop1.get64(reg_index, true);

                                reg64(ui, name, value);
                            }
                        });
                    }

                    reg32(ui, "FCR", last_update.cop1.fcr31.read());
                }
            });

        vec![]
    }
}
