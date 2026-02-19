use egui::RichText;

use crate::ui::colors::Color;

// TODO color enum?

pub struct Text {
    text: String,
    bold: bool,
    color: Color,
    bgcolor: Color,
    reverse: bool,
}

impl Text {
    pub fn new(string: impl AsRef<str>) -> Self {
        Self {
            text: string.as_ref().to_string(),
            bold: false,
            color: Color::Default,
            bgcolor: Color::Default,
            reverse: false,
        }
    }

    pub fn bold(self) -> Self {
        Self { bold: true, ..self }
    }

    pub fn color(self, color: Color) -> Self {
        Self { color, ..self }
    }

    // pub fn bgcolor(self, color: Color) -> Self {
    //     Self {
    //         bgcolor: color,
    //         ..self
    //     }
    // }

    pub fn reverse(self, reverse: bool) -> Self {
        Self { reverse, ..self }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let mut text = RichText::new(self.text).monospace();

        if self.bold {
            text = text.strong();
        }

        if self.color != Color::Default {
            text = text.color(if self.reverse {
                self.bgcolor
            } else {
                self.color
            });
        }

        if self.bgcolor != Color::Default {
            text = text.background_color(self.bgcolor);
        } else if self.reverse {
            text = text.background_color(self.color);
        }

        ui.label(text);
    }
}
