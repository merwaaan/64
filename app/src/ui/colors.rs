use egui::Color32;

// #[derive(Copy, Clone, PartialEq)]
// pub enum Color {
//     Default,
//     Light,
//     //Dark,
//     Active,
//     Success,
//     Warning,
//     Error,
// }

pub const DEFAULT: Color32 = Color32::GRAY;
pub const LIGHT: Color32 = Color32::from_rgb(200, 200, 255);
pub const ACTIVE: Color32 = Color32::from_rgb(255, 143, 183);
pub const SUCCESS: Color32 = Color32::from_rgb(69, 139, 115);
pub const WARNING: Color32 = Color32::from_rgb(255, 209, 80);
pub const ERROR: Color32 = Color32::from_rgb(242, 96, 118);

// impl From<Color> for Color32 {
//     fn from(color: Color) -> Self {
//         const LIGHT_COLOR: Color32 = Color32::from_rgb(200, 200, 255);
//         const ACTIVE_COLOR: Color32 = Color32::from_rgb(255, 143, 183);
//         const SUCCESS_COLOR: Color32 = Color32::from_rgb(69, 139, 115);
//         const WARNING_COLOR: Color32 = Color32::from_rgb(255, 209, 80);
//         const ERROR_COLOR: Color32 = Color32::from_rgb(242, 96, 118);

//         match color {
//             Color::Default => Color32::GRAY,
//             Color::Light => LIGHT_COLOR,
//             //Color::Dark => Color32::DARK_GRAY,
//             Color::Active => ACTIVE_COLOR,
//             Color::Success => SUCCESS_COLOR,
//             Color::Warning => WARNING_COLOR,
//             Color::Error => ERROR_COLOR,
//         }
//     }
// }

pub fn lerp(from: Color32, to: Color32, progress: f64) -> Color32 {
    if progress < 0.0 {
        from
    } else if progress > 1.0 {
        to
    } else {
        let p = progress as f32;
        let ip = 1.0 - p;

        Color32::from_rgb(
            (from.r() as f32 * ip + to.r() as f32 * p) as u8,
            (from.g() as f32 * ip + to.g() as f32 * p) as u8,
            (from.b() as f32 * ip + to.b() as f32 * p) as u8,
        )
    }
}
