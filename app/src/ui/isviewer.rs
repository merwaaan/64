use egui::{Context, Label, ScrollArea, Window};

use crate::{emu::{command::Command, event::Event}, ui::Widget};

#[derive(Default)]
pub struct IsViewerWidget {
    last_update: Option<String>,
}

impl Widget for IsViewerWidget {
    fn update(&mut self, _ctx: &Context, event: &Event) {
        if let Event::IsViewerUpdate(text) = event {
            self.last_update = Some(text.clone());
        }
    }

    fn show(&mut self, ctx: &Context) -> Vec<Command> {
        Window::new("IS Viewer")
            .default_pos([400.0, 800.0])
            .default_width(600.0)
            .show(ctx, |ui| {
                if let Some(text) = &self.last_update {
                    ScrollArea::vertical().show(ui, |ui| {
                        ui.add(Label::new(text).wrap());
                    });
                }
            });

        vec![]
    }
}
