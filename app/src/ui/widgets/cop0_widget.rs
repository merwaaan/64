use std::collections::HashSet;

use crate::{
    command::Command,
    event::Event,
    ui::{
        Data, reg64,
        widgets::{ChildWidget, Widget, WidgetId},
    },
};
use egui::Context;
use n64_core::cop0::Cop0;

#[derive(Default)]
pub struct Cop0Widget {
    id: WidgetId,
    last_update: Option<Cop0>,
}

impl Widget for Cop0Widget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn requested_data(&mut self, _only_if_changed: bool) -> Option<HashSet<Data>> {
        Some(HashSet::from([Data::Cop0]))
    }

    fn update(&mut self, _ctx: &Context, event: &Event) {
        if let Event::Cop0(cop0) = event {
            self.last_update = Some(*cop0);
        }
    }
}

impl ChildWidget for Cop0Widget {
    fn show(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        if let Some(cop0) = &self.last_update {
            for row in 0..16 {
                ui.horizontal(|ui| {
                    for col in 0..2 {
                        let reg_index = row + col * 16;
                        let name = format!("{:>8}", Cop0::reg_name(reg_index));
                        let value = cop0.read(reg_index).get64();

                        reg64(ui, name, value);
                    }
                });
            }
        }

        vec![]
    }
}
