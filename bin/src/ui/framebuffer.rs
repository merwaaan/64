use egui::{ColorImage, Context, RichText, TextureFilter, TextureHandle, TextureOptions};

use crate::{
    emu::{command::Command, event::Event, runner::Runner},
    ui::{SettingUpdate, parse_hex},
};

#[derive(Clone)]
pub struct FramebufferUpdate {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

#[derive(Default)]
pub struct FramebufferWidget {
    last_update: Option<FramebufferUpdate>,

    texture: Option<TextureHandle>,
}

impl FramebufferWidget {
    pub fn update(&mut self, ctx: &Context, event: &Event) {
        if let Event::FramebufferUpdate(update) = event {
            self.last_update = Some(update.clone());

            let image =
                ColorImage::from_rgba_unmultiplied([update.width, update.height], &update.data);

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
                    self.texture = Some(ctx.load_texture("framebuffer", image, options));
                }
            }
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        if let Some(last_update) = &self.last_update {
            ui.label(format!("{}x{}", last_update.width, last_update.height));
        }

        if let Some(texture) = &self.texture {
            ui.image((texture.id(), texture.size_vec2()));
        }
    }
}
