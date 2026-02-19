use egui::{ColorImage, Context, TextureFilter, TextureHandle, TextureOptions, Window};

use crate::{
    emu::{command::Command, event::Event},
    ui::{SettingUpdate, Widget, colors::Color, text::Text},
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

impl Widget for FramebufferWidget {
    fn init(&mut self) -> Vec<Command> {
        vec![Command::SetSetting(SettingUpdate::Framebuffer(Some(())))]
    }

    fn update(&mut self, ctx: &Context, event: &Event) {
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

    fn show(&mut self, ctx: &Context) -> Vec<Command> {
        Window::new("Framebuffer")
            .default_pos([400.0, 1000.0])
            .show(ctx, |ui| {
                if let Some(last_update) = &self.last_update {
                    Text::new(format!("{}x{}", last_update.width, last_update.height))
                        .color(Color::Light)
                        .show(ui);
                }

                if let Some(texture) = &self.texture {
                    ui.image((texture.id(), texture.size_vec2()));
                }
            });

        vec![]
    }
}
