use std::collections::HashSet;

use egui::{ColorImage, Context, TextureFilter, TextureHandle, TextureOptions};

use crate::{
    command::Command,
    event::Event,
    ui::{
        Data, Widget, colors,
        text::Text,
        widgets::{ChildWidget, WidgetId},
    },
};

#[derive(Clone, Debug)]
pub struct FramebufferUpdate {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

#[derive(Default)]
pub struct FramebufferWidget {
    id: WidgetId,
    last_update: Option<FramebufferUpdate>,
    texture: Option<TextureHandle>,
    data_requested: bool,
}

impl Widget for FramebufferWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn requested_data(&mut self, _only_if_changed: bool) -> Option<HashSet<Data>> {
        if !self.data_requested {
            self.data_requested = true;
            Some(HashSet::from([Data::Framebuffer]))
        } else {
            None
        }
    }

    fn update(&mut self, ctx: &Context, event: &Event) {
        if let Event::Framebuffer(update) = event {
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
}

impl ChildWidget for FramebufferWidget {
    fn show(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        if let Some(last_update) = &self.last_update {
            Text::new(format!("{}x{}", last_update.width, last_update.height))
                .color(colors::LIGHT)
                .show(ui);
        }

        if let Some(texture) = &self.texture {
            ui.image((texture.id(), texture.size_vec2()));
        }

        vec![]
    }
}
