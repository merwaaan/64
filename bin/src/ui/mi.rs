use egui::{Context, Window};
use n64::mi::{Interrupt, Mi, Register};
use strum::IntoEnumIterator;

use crate::emu::command::Command;
use crate::ui::colors::Color;
use crate::ui::text::Text;
use crate::{
    emu::event::Event,
    ui::{Widget, reg32},
};

#[derive(Default)]
pub struct MiWidget {
    last_update: Option<Mi>,
}

impl Widget for MiWidget {
    fn update(&mut self, _ctx: &Context, event: &Event) {
        if let Event::MiUpdate(mi) = event {
            self.last_update = Some(*mi);
        }
    }

    fn show(&mut self, ctx: &Context) -> Vec<Command> {
        Window::new("MI")
            .default_pos([400.0, 600.0])
            .show(ctx, |ui| {
                if let Some(mi) = &self.last_update {
                    let mut show_reg = |reg: Register| {
                        reg32(ui, format!("{:>9}", reg), mi.regs[reg as usize]);
                    };

                    Register::iter().for_each(|reg| {
                        show_reg(reg);
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        for interrupt in Interrupt::iter().rev() {
                            ui.horizontal(|ui| {
                                Text::new(format!("{}", interrupt))
                                    .color(if mi.has_pending_interrupt(interrupt) {
                                        if mi.is_interrupt_enabled(interrupt) {
                                            Color::Success
                                        } else {
                                            Color::Warning
                                        }
                                    } else {
                                        Color::Error
                                    })
                                    .show(ui);
                            });
                        }
                    });
                }
            });

        vec![]
    }
}
