use egui::{Context, Window};
use n64_core::si::{Register, Si};
use strum::IntoEnumIterator;

use crate::{
    emu::{command::Command, event::Event},
    ui::{Widget, reg32},
};

#[derive(Default)]
pub struct SiWidget {
    last_update: Option<Si>,
}

impl Widget for SiWidget {
    fn update(&mut self, _ctx: &Context, event: &Event) {
        if let Event::SiUpdate(si) = event {
            self.last_update = Some(*si);
        }
    }

    fn show(&mut self, ctx: &Context) -> Vec<Command> {
        Window::new("SI")
            .default_pos([400.0, 300.0])
            .show(ctx, |ui| {
                if let Some(si) = &self.last_update {
                    let mut show_reg = |reg: Register| {
                        reg32(ui, format!("{:>15}", reg), si.regs[reg as usize]);
                    };

                    Register::iter().for_each(|reg| {
                        show_reg(reg);
                    });
                }
            });

        vec![]
    }
}
