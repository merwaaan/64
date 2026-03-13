use std::collections::HashSet;

use egui::Context;
use n64_core::mi::{Interrupt, Mi};
use strum::IntoEnumIterator;

use crate::{
    command::Command,
    event::Event,
    ui::{
        Data,
        colors::Color,
        reg32,
        text::Text,
        widgets::{ChildWidget, Widget, WidgetId},
    },
};

#[derive(Default)]
pub struct MiWidget {
    id: WidgetId,
    last_update: Option<Mi>,
}

impl Widget for MiWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn requested_data(&mut self, _only_if_changed: bool) -> Option<HashSet<Data>> {
        Some(HashSet::from([Data::Mi]))
    }

    fn update(&mut self, _ctx: &Context, event: &Event) {
        if let Event::Mi(mi) = event {
            self.last_update = Some(*mi);
        }
    }
}

impl ChildWidget for MiWidget {
    fn show(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        if let Some(mi) = &self.last_update {
            reg32(ui, "Mode", mi.regs().mode.raw_value());
            reg32(ui, "Version", mi.regs().version.raw_value());
            reg32(ui, "Interrupts", mi.regs().interrupts.raw_value());
            reg32(ui, "Mask", mi.regs().mask.raw_value());

            ui.separator();

            ui.horizontal(|ui| {
                for interrupt in Interrupt::iter().rev() {
                    ui.horizontal(|ui| {
                        Text::new(format!("{}", interrupt))
                            .color(if mi.is_interrupt_pending(interrupt) {
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

        vec![]
    }
}
