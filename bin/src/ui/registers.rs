use egui::{Context, Window};
use n64::{
    cop0::Cop0,
    registers::{Reg64, Registers},
};

use crate::{
    emu::{command::Command, event::Event},
    ui::{SettingUpdate, Widget, reg64},
};

#[derive(Default)]
pub struct RegistersWidget {
    last_update: Option<RegistersUpdate>,
}

#[derive(Clone, Copy)]
pub struct RegistersUpdate {
    pub cpu_regs: Registers,
    pub cop0_regs: [Reg64; 32],
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
                    reg64(ui, "PC", last_update.cpu_regs.pc as u64);

                    ui.horizontal(|ui| {
                        reg64(ui, "HI", last_update.cpu_regs.mult_hi.get64());
                        reg64(ui, "LO", last_update.cpu_regs.mult_lo.get64());
                    });

                    for row in 0..16 {
                        ui.horizontal(|ui| {
                            for col in 0..2 {
                                let reg_index = row + col * 16;
                                let name = Registers::gpr_name(reg_index);
                                let value = last_update.cpu_regs.gpr[reg_index].get64();

                                reg64(ui, name, value);
                            }
                        });
                    }

                    for row in 0..16 {
                        for col in 0..2 {
                            let reg_index = row + col * 16;
                            let name = Cop0::reg_name(reg_index);
                            let value = last_update.cop0_regs[reg_index].get64();

                            reg64(ui, name, value);
                        }
                    }
                }
            });

        vec![]
    }
}
