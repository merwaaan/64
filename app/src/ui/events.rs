use egui::{Context, ScrollArea, Window};
use n64_core::events::EventType;

use crate::{
    emu::{command::Command, event::Event},
    ui::{Widget, colors::Color, text::Text},
};

#[derive(Default)]
pub struct EventsWidget {
    current_cycle: Option<usize>,
    pending: Vec<(EventType, usize)>,
}

impl Widget for EventsWidget {
    fn update(&mut self, _ctx: &Context, event: &Event) {
        if let Event::CoreEventsUpdate {
            current_cycle,
            pending,
        } = event
        {
            self.current_cycle = Some(*current_cycle);
            self.pending = pending.clone();
        }
    }

    fn show(&mut self, ctx: &Context) -> Vec<Command> {
        Window::new("Events")
            .default_pos([500.0, 200.0])
            .default_width(320.0)
            .default_height(300.0)
            .show(ctx, |ui| {
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
            });

        vec![]
    }
}
