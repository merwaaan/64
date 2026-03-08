use egui::{Context, Window};
use n64_core::sp::Register;
use strum::IntoEnumIterator;

use crate::{
    emu::{command::Command, event::Event},
    ui::{Widget, reg32},
};

#[derive(Default)]
pub struct SpWidget {
    last_update: Option<[u32; 8]>,
}

impl Widget for SpWidget {
    fn update(&mut self, _ctx: &Context, event: &Event) {
        if let Event::SpUpdate(sp) = event {
            self.last_update = Some(*sp);
        }
    }

    fn show(&mut self, ctx: &Context) -> Vec<Command> {
        Window::new("SP")
            .default_pos([1300.0, 500.0])
            .show(ctx, |ui| {
                if let Some(sp_regs) = &self.last_update {
                    let mut show_reg = |reg: Register| {
                        reg32(ui, format!("{:>10}", reg), sp_regs[reg as usize]);
                    };

                    Register::iter().for_each(|reg| {
                        show_reg(reg);
                    });
                }
            });

        vec![]
    }
}
