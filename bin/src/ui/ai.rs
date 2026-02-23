use egui::{Context, Window};
use n64::ai::{Ai, Register};
use strum::IntoEnumIterator;

use crate::{
    emu::{command::Command, event::Event},
    ui::{Widget, reg32},
};

#[derive(Default)]
pub struct AiWidget {
    last_update: Option<Ai>,
}

impl Widget for AiWidget {
    fn update(&mut self, _ctx: &Context, event: &Event) {
        if let Event::AiUpdate(ai) = event {
            self.last_update = Some(*ai);
        }
    }

    fn show(&mut self, ctx: &Context) -> Vec<Command> {
        Window::new("AI")
            .default_pos([400.0, 100.0])
            .show(ctx, |ui| {
                if let Some(ai) = &self.last_update {
                    let mut show_reg = |reg: Register| {
                        reg32(ui, format!("{:>11}", reg), ai.regs[reg as usize]);
                    };

                    Register::iter().for_each(|reg| {
                        show_reg(reg);
                    });
                }
            });

        vec![]
    }
}
