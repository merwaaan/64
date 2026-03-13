use std::collections::HashSet;

use egui::Context;
use n64_core::si::{Register, Si};
use strum::IntoEnumIterator;

use crate::{
    command::Command,
    event::Event,
    ui::{
        Data, reg32,
        widgets::{ChildWidget, Widget, WidgetId},
    },
};

#[derive(Default)]
pub struct SiWidget {
    id: WidgetId,
    last_update: Option<Si>,
}

impl Widget for SiWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn requested_data(&mut self, _only_if_changed: bool) -> Option<HashSet<Data>> {
        Some(HashSet::from([Data::Si]))
    }

    fn update(&mut self, _ctx: &Context, event: &Event) {
        if let Event::Si(si) = event {
            self.last_update = Some(*si);
        }
    }
}

impl ChildWidget for SiWidget {
    fn show(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        if let Some(si) = &self.last_update {
            let mut show_reg = |reg: Register| {
                reg32(ui, format!("{:>14}", reg), si.regs[reg as usize]);
            };

            Register::iter().for_each(|reg| {
                show_reg(reg);
            });
        }

        vec![]
    }
}
