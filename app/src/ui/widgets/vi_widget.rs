use std::collections::HashSet;

use egui::Context;
use n64_core::vi::Vi;

use crate::{
    command::Command,
    event::Event,
    ui::{
        Data, reg32,
        widgets::{ChildWidget, Widget, WidgetId},
    },
};

#[derive(Default)]
pub struct ViWidget {
    id: WidgetId,
    last_update: Option<Vi>,
}

impl Widget for ViWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn requested_data(&mut self, _only_if_changed: bool) -> Option<HashSet<Data>> {
        Some(HashSet::from([Data::Vi]))
    }

    fn update(&mut self, _ctx: &Context, event: &Event) {
        if let Event::Vi(vi) = event {
            self.last_update = Some(*vi);
        }
    }
}

impl ChildWidget for ViWidget {
    fn show(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        if let Some(mi) = &self.last_update {
            reg32(ui, "Control", mi.regs().control.raw_value());
            reg32(ui, "Origin", mi.regs().origin.raw_value());
            reg32(ui, "Width", mi.regs().width.raw_value());
            reg32(ui, "Interrupt", mi.regs().interrupt_line.raw_value());
            reg32(ui, "Current", mi.regs().current_line.raw_value());
            reg32(ui, "Burst", mi.regs().burst.raw_value());
            reg32(ui, "V Total", mi.regs().vertical_total.raw_value());
            reg32(ui, "H Total", mi.regs().horizontal_total.raw_value());
            reg32(ui, "H Leap", mi.regs().horizontal_leap.raw_value());
            reg32(ui, "H Video", mi.regs().horizontal_video.raw_value());
            reg32(ui, "V Video", mi.regs().vertical_video.raw_value());
            reg32(ui, "V Burst", mi.regs().vertical_burst.raw_value());
            reg32(ui, "H Scale", mi.regs().horizontal_scale.raw_value());
            reg32(ui, "V Scale", mi.regs().vertical_scale.raw_value());
        }

        vec![]
    }
}
