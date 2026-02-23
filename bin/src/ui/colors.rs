use egui::Color32;

#[derive(Copy, Clone, PartialEq)]
pub enum Color {
    Default,
    Light,
    //Dark,
    Active,
    Success,
    Warning,
    Error,
}

impl From<Color> for Color32 {
    fn from(color: Color) -> Self {
        match color {
            Color::Default => Color32::GRAY,
            Color::Light => LIGHT_COLOR,
            //Color::Dark => Color32::DARK_GRAY,
            Color::Active => ACTIVE_COLOR,
            Color::Success => SUCCESS_COLOR,
            Color::Warning => WARNING_COLOR,
            Color::Error => ERROR_COLOR,
        }
    }
}

const LIGHT_COLOR: Color32 = Color32::from_rgb(200, 200, 255);
const ACTIVE_COLOR: Color32 = Color32::from_rgb(255, 143, 183);
const SUCCESS_COLOR: Color32 = Color32::from_rgb(69, 139, 115);
const WARNING_COLOR: Color32 = Color32::from_rgb(255, 209, 80);
const ERROR_COLOR: Color32 = Color32::from_rgb(242, 96, 118);
