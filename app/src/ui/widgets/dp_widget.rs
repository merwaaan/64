use std::{collections::HashSet, sync::Arc};

use crate::{
    command::Command,
    event::Event,
    ui::{
        Data, colors, reg32,
        text::Text,
        widgets::{ChildWidget, Widget, WidgetId},
    },
};
use egui::{
    Color32, ColorImage, Context, Rect, Scene, ScrollArea, TextureFilter, TextureHandle,
    TextureOptions, vec2,
};
use n64_core::{dp::rgba5551_to_8888, rendering::video::Frame};

#[derive(Clone, Debug)]
pub struct DpUpdate {
    pub regs: [u32; 8],
    pub tmem: [u8; 0x1000],

    // TODO move out
    pub atlas_texture: Arc<Frame>,
}

pub struct DpWidget {
    id: WidgetId,
    last_update: Option<DpUpdate>,
    texture: Option<TextureHandle>,
    atlas_rect: Rect,
}

impl Default for DpWidget {
    fn default() -> Self {
        Self {
            id: WidgetId::default(),
            last_update: None,
            texture: None,
            atlas_rect: Rect::ZERO,
        }
    }
}

impl Widget for DpWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn requested_data(&mut self, _only_if_changed: bool) -> Option<HashSet<Data>> {
        Some(HashSet::from([Data::Dp]))
    }

    fn update(&mut self, ctx: &Context, event: &Event) {
        if let Event::Dp(update) = event {
            self.last_update = Some(update.clone());

            let image = ColorImage::from_rgba_unmultiplied(
                [update.atlas_texture.width, update.atlas_texture.height],
                &update.atlas_texture.rgba,
            );

            let options = TextureOptions {
                magnification: TextureFilter::Nearest,
                minification: TextureFilter::Nearest,
                mipmap_mode: None,
                ..Default::default()
            };

            match &mut self.texture {
                Some(texture) => {
                    texture.set(image, options);
                }
                None => {
                    self.texture = Some(ctx.load_texture("atlas", image, options));
                }
            }
        }
    }
}

impl ChildWidget for DpWidget {
    fn show(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        // TODO clean up UI

        // Registers

        if let Some(update) = &self.last_update {
            let mut show_reg = |i| {
                reg32(ui, format!("{:>10}", i), update.regs[i]);
            };

            update.regs.iter().enumerate().for_each(|(i, _)| {
                show_reg(i);
            });

            // TMEM

            ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                for (chunk_index, chunk) in update.tmem.chunks(16).enumerate() {
                    let addr = (chunk_index * 16) as u32;

                    ui.horizontal(|ui| {
                        ui.style_mut().spacing.item_spacing = vec2(0.0, 0.0);

                        Text::new(format!("{:08X}", addr))
                            .color(colors::LIGHT)
                            .show(ui);

                        ui.add_space(4.0); // TODO do it once?

                        for word in chunk.chunks(4) {
                            if addr < 0x800 {
                                Text::new(format!(
                                    "{:02X}{:02X}{:02X}{:02X}",
                                    word[0], word[1], word[2], word[3]
                                ))
                                .show(ui);
                            } else {
                                // Palettes: color each 16-bit color

                                let color1 = rgba5551_to_8888(word[0], word[1]);

                                Text::new(format!("{:02X}{:02X}", word[0], word[1]))
                                    .color(Color32::from_rgb(color1[0], color1[1], color1[2]))
                                    .show(ui);

                                let color2 = rgba5551_to_8888(word[2], word[3]);

                                Text::new(format!("{:02X}{:02X}", word[2], word[3]))
                                    .color(Color32::from_rgb(color2[0], color2[1], color2[2]))
                                    .show(ui);
                            }

                            ui.add_space(4.0);
                        }
                    });
                }
            });
        }

        // Tile atlas TODO move to separate widget

        if let Some(texture) = &self.texture {
            // Scene::new().zoom_range(0.0..=10.0).show(
            //     ui,
            //     &mut self.atlas_rect,
            //     |ui: &mut egui::Ui| {
            ui.image((texture.id(), texture.size_vec2()));
            //     },
            // );
        }

        vec![]
    }
}
