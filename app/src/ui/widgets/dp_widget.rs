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
use n64_core::{
    dp::rgba5551_to_8888,
    rendering::{tile_cache::Tile, video::Frame},
};

#[derive(Clone, Debug)]
pub struct DpUpdate {
    pub regs: [u32; 8],
    pub tmem: [u8; 0x1000],
    // TODO move out
    pub tiles: Vec<(u64, Tile)>,
}

pub struct DpWidget {
    id: WidgetId,
    last_update: Option<DpUpdate>,

    tile_textures: Vec<(u64, TextureHandle)>,
    //atlas_rect: Rect,
}

impl Default for DpWidget {
    fn default() -> Self {
        Self {
            id: WidgetId::default(),
            last_update: None,
            tile_textures: Vec::new(),
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

            // TODO only new tiles?

            for tile_index in self.tile_textures.len()..update.tiles.len() {
                let tile = &update.tiles[tile_index];

                let image = ColorImage::from_rgba_unmultiplied(
                    [tile.1.width as usize, tile.1.height as usize],
                    &tile.1.rgba,
                );

                let options = TextureOptions {
                    magnification: TextureFilter::Nearest,
                    minification: TextureFilter::Nearest,
                    mipmap_mode: None,
                    ..Default::default()
                };

                self.tile_textures.push((
                    tile.0,
                    ctx.load_texture(format!("tile_{}", tile_index), image, options),
                ));

                // match self.tile_textures.get_mut(tile_index) {
                //     Some(texture) => {
                //         texture.set(image, options);
                //     }
                //     None => {
                //         self.tile_textures[tile_index] =
                //             ctx.load_texture(format!("tile_{}", tile_index), image, options);
                //     }
                // }
            }
        }
    }
}

impl ChildWidget for DpWidget {
    fn show(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        // TODO clean up UI

        // Registers

        if let Some(update) = &self.last_update {
            const REG_NAMES: [&str; 8] = [
                "Start",
                "End",
                "Current",
                "Status",
                "Clock",
                "Buf busy",
                "Pipe busy",
                "Tmem busy",
            ];

            for (i, name) in REG_NAMES.iter().enumerate() {
                reg32(ui, format!("{:>9}", name), update.regs[i]);
            }

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

        Text::new(format!("{} tiles", self.tile_textures.len())).show(ui);

        for (tile_index, tile_texture) in self.tile_textures.iter().enumerate() {
            ui.horizontal(|ui| {
                Text::new(format!("#{} {}", tile_index, tile_texture.0)).show(ui);
                ui.image((tile_texture.1.id(), tile_texture.1.size_vec2()));
            });
        }

        vec![]
    }
}
