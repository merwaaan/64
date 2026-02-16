use egui::{Color32, RichText, vec2};

use crate::{
    emu::{command::Command, event::Event, runner::Runner},
    ui::{SettingUpdate, parse_hex},
};

#[derive(Clone, Copy)]
pub struct MemorySettings {
    pub address: u32,
    pub rows: usize, // 16 bytes per row
}

impl Default for MemorySettings {
    fn default() -> Self {
        Self {
            address: 0xA010_0000,
            rows: 8,
        }
    }
}

pub struct MemoryUpdate {
    pub base_address: u32,
    pub data: Vec<u8>,
}

#[derive(Default)]
pub struct MemoryWidget {
    pub settings: MemorySettings,

    base_address: u32,
    data: Vec<u8>,

    address_input: String,
}

impl MemoryWidget {
    pub fn update(&mut self, event: &Event) {
        if let Event::MemoryUpdate(update) = event {
            self.base_address = update.base_address;
            self.data = update.data.clone();
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, runner: &mut Option<Runner>) {
        let mut settings_changed = false;

        ui.horizontal(|ui| {
            // Address input

            ui.label("Address:");

            let prev_address_input = self.address_input.clone();
            ui.text_edit_singleline(&mut self.address_input);

            if prev_address_input != self.address_input
                && let Some(address) = parse_hex(&self.address_input)
            {
                self.settings.address = address as u32;
                settings_changed = true;
            }

            // Up/Down buttons

            if ui.button("⬆").clicked() {
                self.settings.address = self
                    .settings
                    .address
                    .wrapping_sub(self.settings.rows as u32 * 16);
                settings_changed = true;
            }

            if ui.button("⬇").clicked() {
                self.settings.address = self
                    .settings
                    .address
                    .wrapping_add(self.settings.rows as u32 * 16);
                settings_changed = true;
            }
        });

        let address_target = parse_hex(&self.address_input).map(|addr| addr as u32);

        for (chunk_index, chunk) in self.data.chunks(16).enumerate() {
            let offset = (chunk_index * 16) as u32;

            let addr = self.base_address + offset;

            ui.horizontal(|ui| {
                ui.style_mut().spacing.item_spacing = vec2(0.0, 0.0);

                ui.label(RichText::new(format!("{:08X}", addr)).monospace().strong());
                //ui.add_space(4.0);

                for (byte_index, byte) in chunk.iter().enumerate() {
                    if byte_index % 4 == 0 {
                        ui.add_space(4.0);
                    }

                    let mut text = RichText::new(format!("{:02X}", byte)).monospace();

                    if let Some(address_target) = address_target
                        && address_target == addr + byte_index as u32
                    {
                        text = text.color(Color32::from_rgb(0, 255, 0));
                    }

                    ui.label(text);
                }
            });
        }

        // Update the settings if they changed

        if settings_changed && let Some(runner) = runner {
            runner.send_command(Command::SetSetting(SettingUpdate::Memory(Some(
                self.settings,
            ))));
        }
    }
}
