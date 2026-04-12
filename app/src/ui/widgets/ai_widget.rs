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
            reg32(ui, "DMA RAM address", update.ai.dma_ram_address);
            reg32(ui, "DMA length", update.ai.dma_length);
            reg32(ui, "DMA enabled", update.ai.dma_enabled as u32);
            reg32(ui, "Status", update.ai.status.raw_value());

            ui.horizontal(|ui| {
                reg32(ui, "Dac rate", update.ai.dac_rate);

                Text::new(format!("{} Hz", update.ai.sample_rate())).show(ui);
            });

            ui.separator();

            ui.horizontal(|ui| {
                Text::new("Queued samples").show(ui);
                Text::new(format!("{}", update.queued_samples)).show(ui);
            });
        }

        vec![]
    }
}
