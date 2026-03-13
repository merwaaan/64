use std::collections::HashSet;

use egui::Context;
use n64_core::pi::{Pi, Register};
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
pub struct PiWidget {
    id: WidgetId,
    last_update: Option<Pi>,
}

impl Widget for PiWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn requested_data(&mut self, _only_if_changed: bool) -> Option<HashSet<Data>> {
        Some(HashSet::from([Data::Pi]))
    }

    fn update(&mut self, _ctx: &Context, event: &Event) {
        if let Event::Pi(pi) = event {
            self.last_update = Some(*pi);
        }
    }
}

impl ChildWidget for PiWidget {
    fn show(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        if let Some(pi) = &self.last_update {
            let mut show_reg = |reg: Register| {
                reg32(ui, format!("{:>8}", reg), pi.regs[reg as usize]);
            };

            Register::iter().for_each(|reg| {
                show_reg(reg);
            });
        }

        vec![]
    }
}
