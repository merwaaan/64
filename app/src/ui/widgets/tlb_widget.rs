use std::collections::HashSet;

use egui::{Context, ScrollArea};
use n64_core::tlb::Tlb;

use crate::{
    command::Command,
    event::Event,
    ui::{
        Data,
        widgets::{ChildWidget, Widget, WidgetId},
    },
};

#[derive(Default)]
pub struct TlbWidget {
    id: WidgetId,
    last_update: Option<Tlb>,
}

impl Widget for TlbWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn requested_data(&mut self, _only_if_changed: bool) -> Option<HashSet<Data>> {
        Some(HashSet::from([Data::Tlb]))
    }

    fn update(&mut self, _ctx: &Context, event: &Event) {
        if let Event::Tlb(entries) = event {
            self.last_update = Some(*entries);
        }
    }
}

impl ChildWidget for TlbWidget {
    fn show(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        if let Some(entries) = &self.last_update {
            ScrollArea::vertical().show(ui, |ui| {
                for index in 0..32 {
                    let entry = entries.read(index as u32);
                    ui.collapsing(
                        format!(
                            "#{index:02} VPN2={:011X} ASID={:02X} G={} MASK={:03X}",
                            entry.vpn2(),
                            entry.asid(),
                            if entry.global() { "Y" } else { "N" },
                            entry.mask(),
                        ),
                        |ui| {
                            ui.monospace(format!(
                                "Lo0 PFN={:05X} C={} V={} D={}",
                                entry.page_pfn(0),
                                entry.page_cache(0),
                                entry.page_valid(0) as u8,
                                entry.page_writable(0) as u8,
                            ));
                            ui.monospace(format!(
                                "Lo1 PFN={:05X} C={} V={} D={}",
                                entry.page_pfn(1),
                                entry.page_cache(1),
                                entry.page_valid(1) as u8,
                                entry.page_writable(1) as u8,
                            ));
                        },
                    );
                }
            });
        }

        vec![]
    }
}
