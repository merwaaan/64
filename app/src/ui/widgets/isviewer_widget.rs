use std::collections::HashSet;

use egui::{Context, Label, ScrollArea};

use crate::{
    command::Command,
    event::Event,
    ui::{
        Data, Widget,
        widgets::{ChildWidget, WidgetId},
    },
};

#[derive(Default)]
pub struct IsViewerWidget {
    id: WidgetId,
    last_update: Option<String>,
}

impl Widget for IsViewerWidget {
    fn id(&self) -> super::WidgetId {
        self.id
    }

    fn requested_data(&mut self, _only_if_changed: bool) -> Option<HashSet<Data>> {
        Some(HashSet::from([Data::IsViewer]))
    }

    fn update(&mut self, _ctx: &Context, event: &Event) {
        if let Event::IsViewer(text) = event {
            self.last_update = Some(text.clone());
        }
    }
}

impl ChildWidget for IsViewerWidget {
    fn show(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        if let Some(text) = &self.last_update {
            ScrollArea::vertical().show(ui, |ui| {
                ui.monospace(text);
            });
        }

        vec![]
    }
}
