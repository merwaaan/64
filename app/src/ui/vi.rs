use n64_core::vi::{Register, Vi};
use egui::{Context, Window};
use strum::IntoEnumIterator;

use crate::{
    emu::{command::Command, event::Event},
    ui::{Widget, reg32},
};

#[derive(Default)]
pub struct ViWidget {
    last_update: Option<Vi>,
}

impl Widget for ViWidget {
    fn update(&mut self, _ctx: &Context, event: &Event) {
        if let Event::ViUpdate(vi) = event {
            self.last_update = Some(*vi);
        }
    }

    fn show(&mut self, ctx: &Context) -> Vec<Command> {
        Window::new("VI").default_pos([0.0, 800.0]).show(ctx, |ui| {
            if let Some(mi) = &self.last_update {
                let mut show_reg = |reg: Register| {
                    reg32(ui, format!("{:>17}", reg), mi.regs[reg as usize]);
                };

                Register::iter().for_each(|reg| {
                    show_reg(reg);
                });
            }
        });

        vec![]
    }
}
