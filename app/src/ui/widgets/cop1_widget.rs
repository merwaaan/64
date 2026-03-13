use std::collections::HashSet;

use crate::{
    command::Command,
    event::Event,
    ui::{
        Data, reg32, reg64,
        widgets::{ChildWidget, Widget, WidgetId},
    },
};
use egui::Context;
use n64_core::{cop1::Cop1, registers::Registers};

#[derive(Default)]
pub struct Cop1Widget {
    id: WidgetId,
    last_update: Option<Cop1>,
}

impl Widget for Cop1Widget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn requested_data(&mut self, _only_if_changed: bool) -> Option<HashSet<Data>> {
        Some(HashSet::from([Data::Cop1]))
    }

    fn update(&mut self, _ctx: &Context, event: &Event) {
        if let Event::Cop1(cop1) = event {
            self.last_update = Some(*cop1);
        }
    }
}

impl ChildWidget for Cop1Widget {
    fn show(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        if let Some(cop1) = &self.last_update {
            for row in 0..16 {
                ui.horizontal(|ui| {
                    for col in 0..2 {
                        let reg_index = row + col * 16;
                        let name = format!("{:>3}", Registers::fpr_name(reg_index));
                        let value = cop1.get64(reg_index, true);

                        reg64(ui, name, value);
                    }
                });
            }

            reg32(ui, "FCR", cop1.fcr31.read());
        }

        vec![]
    }
}
