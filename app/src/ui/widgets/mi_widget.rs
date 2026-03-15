use std::collections::HashSet;

use egui::Context;
use n64_core::mi::{Interrupt, Mi};
use strum::IntoEnumIterator;

use crate::{
    command::Command,
    event::Event,
    ui::{
        Data, colors, reg32,
        text::Text,
        widgets::{ChildWidget, Widget, WidgetId},
    },
};

#[derive(Default)]
pub struct MiWidget {
    id: WidgetId,
    last_update: Option<Mi>,

    // Last time each interrupt was active, to fade them out progressively
    // TODO not really working, we miss most interrupts! collect them in the core thread?
    last_interrupt_time: [f64; 6],
}

impl Widget for MiWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn requested_data(&mut self, _only_if_changed: bool) -> Option<HashSet<Data>> {
        Some(HashSet::from([Data::Mi]))
    }

    fn update(&mut self, ctx: &Context, event: &Event) {
        if let Event::Mi(mi) = event {
            self.last_update = Some(*mi);

            let now = ctx.input(|i| i.time);

            for (index, interrupt) in Interrupt::iter().enumerate() {
                if mi.is_interrupt_pending(interrupt) && mi.is_interrupt_enabled(interrupt) {
                    self.last_interrupt_time[index] = now;
                }
            }
        }
    }
}

const INTERRUPT_FADE_TIME: f64 = 1.0;

impl ChildWidget for MiWidget {
    fn show(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        if let Some(mi) = &self.last_update {
            reg32(ui, "Mode", mi.regs().mode.raw_value());
            reg32(ui, "Version", mi.regs().version.raw_value());
            reg32(ui, "Interrupts", mi.regs().interrupts.raw_value());
            reg32(ui, "Mask", mi.regs().mask.raw_value());

            ui.separator();

            ui.horizontal(|ui| {
                let now = ui.ctx().input(|i| i.time);

                for (index, interrupt) in Interrupt::iter().enumerate().rev() {
                    let color = {
                        let state_color = if mi.is_interrupt_pending(interrupt) {
                            if mi.is_interrupt_enabled(interrupt) {
                                colors::SUCCESS
                            } else {
                                colors::WARNING
                            }
                        } else {
                            colors::ERROR
                        };

                        let fade_progress =
                            (now - self.last_interrupt_time[index]) / INTERRUPT_FADE_TIME;

                        colors::lerp(colors::SUCCESS, state_color, fade_progress)
                    };

                    Text::new(format!("{}", interrupt)).color(color).show(ui);
                }
            });
        }

        vec![]
    }
}
