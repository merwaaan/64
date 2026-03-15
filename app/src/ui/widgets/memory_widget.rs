use std::collections::HashSet;

use egui::{Context, vec2};
use n64_core::{
    ai::AiLocation,
    cart::CartLocation,
    dp::DpLocation,
    mi::MiLocation,
    pi::PiLocation,
    pif::PifRamLocation,
    ram::{RamInterfaceLocation, RamLocation, RamRegsLocation},
    si::SiLocation,
    sp::{SpMemLocation, SpRegsLocation},
    vi::ViLocation,
};

use crate::{
    command::Command,
    event::Event,
    ui::{
        Data, Widget, colors, parse_hex,
        text::Text,
        widgets::{ChildWidget, WidgetId},
    },
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct MemorySettings {
    pub address: u32,
    pub rows: usize, // 16 bytes per row
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct MemoryUpdate {
    pub base_address: u32,
    pub data: Vec<Option<u8>>,
}

pub struct MemoryWidget {
    id: WidgetId,
    settings: MemorySettings,
    settings_changed: bool,
    last_update: Option<MemoryUpdate>,
    address_input: String,
}
impl Default for MemoryWidget {
    fn default() -> Self {
        Self {
            id: WidgetId::default(),
            settings: MemorySettings {
                address: 0x0400_0FC0,
                rows: 8,
            },
            settings_changed: true,
            last_update: None,
            address_input: String::new(),
        }
    }
}

impl Widget for MemoryWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn requested_data(&mut self, _only_if_changed: bool) -> Option<HashSet<Data>> {
        if self.settings_changed {
            self.settings_changed = false;
            Some(HashSet::from([Data::Memory(self.settings.clone())]))
        } else {
            None
        }
    }

    fn update(&mut self, _ctx: &Context, event: &Event) {
        if let Event::Memory(update) = event {
            self.last_update = Some(update.clone());
        }
    }
}

impl ChildWidget for MemoryWidget {
    fn show(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        let mut commands = Vec::new();

        ui.horizontal(|ui| {
            // TODO virtual/physical checkbox?

            // Address input

            Text::new("Address").show(ui);

            let prev_address_input = self.address_input.clone();
            ui.text_edit_singleline(&mut self.address_input);

            if prev_address_input != self.address_input
                && let Some(address) = parse_hex(&self.address_input)
            {
                self.settings.address = address as u32;
                self.settings_changed = true;
            }

            // Quick jump

            const REGIONS: &[(&str, u32)] = &[
                ("RAM", RamLocation::START),
                ("RAM registers", RamRegsLocation::START),
                ("Signal processor DMEM", SpMemLocation::START),
                ("Signal processor IMEM", SpMemLocation::START + 0x1000),
                ("Signal processor registers", SpRegsLocation::START),
                ("Display processor", DpLocation::START),
                ("MIPS interface", MiLocation::START),
                ("Video interface", ViLocation::START),
                ("Audio interface", AiLocation::START),
                ("Peripheral interface", PiLocation::START),
                ("RAM interface", RamInterfaceLocation::START),
                ("Serial interface", SiLocation::START),
                ("Cartridge", CartLocation::START),
                ("Pif RAM", PifRamLocation::START),
            ];

            ui.menu_button("Jump to...", |ui| {
                for (name, address) in REGIONS {
                    if ui
                        .selectable_label(self.settings.address == *address, *name)
                        .clicked()
                    {
                        self.settings.address = *address;
                        self.address_input = format!("{:08X}", address);
                        self.settings_changed = true;
                        ui.close();
                    }
                }
            });

            // Up/Down buttons

            if ui.button("⬆").clicked() {
                self.settings.address = self
                    .settings
                    .address
                    .wrapping_sub(self.settings.rows as u32 * 16);
                self.settings_changed = true;
            }

            if ui.button("⬇").clicked() {
                self.settings.address = self
                    .settings
                    .address
                    .wrapping_add(self.settings.rows as u32 * 16);
                self.settings_changed = true;
            }
        });

        ui.separator();

        // TODO use ScrollArea::show_rows to virtualize?

        let address_target = parse_hex(&self.address_input).map(|addr| addr as u32);

        if let Some(last_update) = &self.last_update {
            for (chunk_index, chunk) in last_update.data.chunks(16).enumerate() {
                let offset = (chunk_index * 16) as u32;

                let addr = last_update.base_address + offset;

                ui.horizontal(|ui| {
                    ui.style_mut().spacing.item_spacing = vec2(0.0, 0.0);

                    Text::new(format!("{:08X}", addr))
                        .color(colors::LIGHT)
                        .show(ui);

                    for (byte_index, byte) in chunk.iter().enumerate() {
                        if byte_index % 4 == 0 {
                            ui.add_space(4.0);
                        }

                        let mut text = Text::new(
                            byte.map(|b| format!("{:02X}", b))
                                .unwrap_or("??".to_string()),
                        );

                        // TODO not working?
                        if let Some(address_target) = address_target
                            && address_target == addr + byte_index as u32
                        {
                            text = text.color(colors::ACTIVE);
                        }

                        text.show(ui);
                    }
                });
            }
        }

        commands
    }
}
