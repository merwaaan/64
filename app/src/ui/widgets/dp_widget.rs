use std::collections::HashSet;

use crate::{
    command::Command,
    event::Event,
    ui::{
        Data, reg32,
        widgets::{ChildWidget, Widget, WidgetId},
    },
};
use egui::Context;

#[derive(Default)]
pub struct DpWidget {
    id: WidgetId,
    last_update: Option<[u32; 8]>,
}

impl Widget for DpWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn requested_data(&mut self, _only_if_changed: bool) -> Option<HashSet<Data>> {
        Some(HashSet::from([Data::Dp]))
    }

    fn update(&mut self, _ctx: &Context, event: &Event) {
        if let Event::Dp(dp) = event {
            self.last_update = Some(*dp);
        }
    }
}

impl ChildWidget for DpWidget {
    fn show(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        if let Some(sp_regs) = &self.last_update {
            let mut show_reg = |i| {
                reg32(ui, format!("{:>10}", i), sp_regs[i]);
            };

            sp_regs.iter().enumerate().for_each(|(i, _)| {
                show_reg(i);
            });
        }

        vec![]
    }
}
