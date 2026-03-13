use std::collections::HashSet;

use egui::{Context, ScrollArea};
use n64_core::events::EventType;

use crate::{
    command::Command,
    event::Event,
    ui::{
        Data,
        colors::Color,
        text::Text,
        widgets::{ChildWidget, Widget, WidgetId},
    },
};

#[derive(Default)]
pub struct EventsWidget {
    id: WidgetId,
    current_cycle: Option<usize>,
    pending: Vec<(EventType, usize)>,
}

impl Widget for EventsWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn requested_data(&mut self, _only_if_changed: bool) -> Option<HashSet<Data>> {
        Some(HashSet::from([Data::Events]))
    }

    fn update(&mut self, _ctx: &Context, event: &Event) {
        if let Event::Events {
            current_cycle,
            pending,
        } = event
        {
            self.current_cycle = Some(*current_cycle);
            self.pending = pending.clone();
        }
    }
}

impl ChildWidget for EventsWidget {
    fn show(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        if let Some(cycle) = self.current_cycle {
            ui.horizontal(|ui| {
                Text::new("Cycle").color(Color::Light).show(ui);
                Text::new(format!("{}", cycle)).show(ui);
            });
            ui.separator();
        }
        ScrollArea::vertical()
            .stick_to_bottom(false)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for (event_type, scheduled_cycle) in &self.pending {
                    ui.horizontal(|ui| {
                        Text::new(format!("{:?}", event_type))
                            .color(Color::Active)
                            .show(ui);
                        Text::new(format!("@ {}", scheduled_cycle))
                            .color(Color::Light)
                            .show(ui);
                    });
                }
            });
        vec![]
    }
}
