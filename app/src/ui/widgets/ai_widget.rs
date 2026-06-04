use std::collections::HashSet;

use egui::Context;
use n64_core::ai::Ai;

use crate::{
    command::Command,
    event::Event,
    ui::{
        Data, reg32,
        text::Text,
        widgets::{ChildWidget, Widget, WidgetId},
    },
};

#[derive(Clone, Debug)]
pub struct AiUpdate {
    pub ai: Ai,
    pub queued_samples: usize,
}

#[derive(Default)]
pub struct AiWidget {
    id: WidgetId,
    last_update: Option<AiUpdate>,
}

impl Widget for AiWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn requested_data(&mut self, _only_if_changed: bool) -> Option<HashSet<Data>> {
        Some(HashSet::from([Data::Ai]))
    }

    fn update(&mut self, _ctx: &Context, event: &Event) {
        if let Event::Ai(update) = event {
            self.last_update = Some(update.clone());
        }
    }
}

impl ChildWidget for AiWidget {
    fn show(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        if let Some(update) = &self.last_update {
            // Registers

            reg32(
                ui,
                "DMA RAM address",
                update.ai.regs.dma_ram_address.raw_value(),
            );

            reg32(ui, "DMA length", update.ai.regs.dma_length.raw_value());

            reg32(ui, "DMA enabled", update.ai.regs.control.raw_value());

            reg32(ui, "Status", update.ai.regs.status.raw_value());

            ui.horizontal(|ui| {
                reg32(ui, "Dac rate", update.ai.regs.dac_rate.raw_value());

                Text::new(format!("{} Hz", update.ai.sample_rate())).show(ui);
            });

            reg32(ui, "Bit rate", update.ai.regs.bit_rate.raw_value());

            ui.separator();

            // Samples

            ui.horizontal(|ui| {
                Text::new("Queued samples").show(ui);
                Text::new(format!("{}", update.queued_samples)).show(ui);
            });
        }

        vec![]
    }
}
