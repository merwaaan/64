use egui::{Context, Window, vec2};

use crate::{
    emu::{command::Command, event::Event},
    ui::{SettingUpdate, Widget, colors::Color, parse_hex, text::Text},
};

#[derive(Clone, Copy)]
pub struct MemorySettings {
    pub address: u32,
    pub rows: usize, // 16 bytes per row
}

// TODO default MemoryWidget?
impl Default for MemorySettings {
    fn default() -> Self {
        Self {
            address: 0x0000_0000,
            rows: 8,
        }
    }
}

#[derive(Clone)]
pub struct MemoryUpdate {
    pub base_address: u32,
    pub data: Vec<Option<u8>>,
}

#[derive(Default)]
pub struct MemoryWidget {
    pub settings: MemorySettings,

    last_update: Option<MemoryUpdate>,

    address_input: String,
}

impl Widget for MemoryWidget {
    fn init(&mut self) -> Vec<Command> {
        vec![Command::SetSetting(SettingUpdate::Memory(Some(
            self.settings,
        )))]
    }

    fn update(&mut self, _ctx: &Context, event: &Event) {
        if let Event::MemoryUpdate(update) = event {
            self.last_update = Some(update.clone());
        }
    }

    fn show(&mut self, ctx: &Context) -> Vec<Command> {
        let mut commands = Vec::new();

        Window::new("Memory")
            .default_pos([1600.0, 100.0])
            .show(ctx, |ui| {
                let mut settings_changed = false;

                ui.horizontal(|ui| {
                    // Address input

                    Text::new("Address").show(ui);

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

                ui.separator();

                let address_target = parse_hex(&self.address_input).map(|addr| addr as u32);

                if let Some(last_update) = &self.last_update {
                    for (chunk_index, chunk) in last_update.data.chunks(16).enumerate() {
                        let offset = (chunk_index * 16) as u32;

                        let addr = last_update.base_address + offset;

                        ui.horizontal(|ui| {
                            ui.style_mut().spacing.item_spacing = vec2(0.0, 0.0);

                            Text::new(format!("{:08X}", addr))
                                .color(Color::Light)
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
                                    text = text.color(Color::Active);
                                }

                                text.show(ui);
                            }
                        });
                    }

                    // Update the settings if they changed

                    if settings_changed {
                        commands.push(Command::SetSetting(SettingUpdate::Memory(Some(
                            self.settings,
                        ))));
                    }
                }
            });

        commands
    }
}
