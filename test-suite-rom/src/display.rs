use alloc::vec::Vec;
use anyhow::{Result, anyhow};
use arbitrary_int::*;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
};
use embedded_text::{
    TextBox,
    style::{HeightMode, TextBoxStyleBuilder, VerticalOverdraw},
};
use n64_specs::{
    color::{RGBA5551, RGBA8888},
    vi,
};

use crate::io;

pub const WIDTH: u32 = 320;
pub const HEIGHT: u32 = 240;
pub const PIXELS: usize = (WIDTH * HEIGHT) as usize;

const MARGIN: u32 = 8;
const PROGRESS_HEIGHT: u32 = 16;
const TEXT_BOTTOM: u32 = HEIGHT - MARGIN - PROGRESS_HEIGHT;

pub const WHITE: RGBA5551 = RGBA5551::from_rgba(0xFF, 0xFF, 0xFF, 0xFF);
pub const SUCCESS: RGBA5551 = RGBA5551::from_rgba(0x4C, 0xAF, 0x50, 0xFF);
pub const WARNING: RGBA5551 = RGBA5551::from_rgba(0xFF, 0x98, 0x00, 0xFF);
pub const ERROR: RGBA5551 = RGBA5551::from_rgba(0xEF, 0x53, 0x50, 0xFF);

pub struct Display {
    // We use a 16-bit framebuffer to save memory
    buffer: Vec<u16>,
    text_cursor_y: u32,
}

impl Default for Display {
    fn default() -> Self {
        let buffer = alloc::vec![WHITE.raw_value(); PIXELS];

        // TODO use specific reg for each write instead of base + offset
        // TODO does the IPL do that?

        let vi_reg_base = io::uncached_ptr::<u32>(vi::START);

        // TODO a bit glitchy, was better before tweaking the settings

        unsafe {
            vi_reg_base.add(vi::Control::INDEX).write_volatile(
                vi::Control::default()
                    .with_color_mode(vi::ColorMode::Rgba5551)
                    .with_gamma_dither(true)
                    .with_gamma(true)
                    .with_antialias_mode(vi::AntiAliasingMode::Resample)
                    .with_pixel_advance(u4::new(1))
                    .raw_value(),
            );

            vi_reg_base
                .add(vi::Origin::INDEX)
                .write_volatile(buffer.as_ptr() as u32);

            vi_reg_base.add(vi::Width::INDEX).write_volatile(WIDTH);

            vi_reg_base.add(vi::InterruptLine::INDEX).write_volatile(2);

            vi_reg_base
                .add(vi::Burst::INDEX)
                .write_volatile(vi::BURST_NTSC);

            vi_reg_base
                .add(vi::VerticalTotal::INDEX)
                .write_volatile(vi::VERTICAL_TOTAL_NTSC_PROGRESSIVE);

            vi_reg_base
                .add(vi::HorizontalTotal::INDEX)
                .write_volatile(vi::HORIZONTAL_TOTAL_NTSC);

            vi_reg_base
                .add(vi::HorizontalTotalLeap::INDEX)
                .write_volatile(vi::HORIZONTAL_TOTAL_LEAP_NTSC);

            vi_reg_base
                .add(vi::HorizontalVideo::INDEX)
                .write_volatile(vi::HORIZONTAL_VIDEO_NTSC);

            vi_reg_base
                .add(vi::VerticalVideo::INDEX)
                .write_volatile(vi::VERTICAL_VIDEO_NTSC);

            vi_reg_base
                .add(vi::VerticalBurst::INDEX)
                .write_volatile(vi::VERTICAL_BURST_NTSC);

            vi_reg_base
                .add(vi::HorizontalScale::INDEX)
                .write_volatile(vi::horizontal_scale_from_width(WIDTH).raw_value());

            vi_reg_base
                .add(vi::VerticalScale::INDEX)
                .write_volatile(vi::vertical_scale_from_height(HEIGHT).raw_value());
        }

        Self {
            buffer,
            text_cursor_y: MARGIN,
        }
    }
}

impl Display {
    pub fn fill(&mut self, color: RGBA5551) {
        let buffer_uncached = self.uncached_buffer_ptr();

        unsafe {
            for i in 0..PIXELS {
                buffer_uncached.add(i).write_volatile(color.raw_value());
            }
        }
    }

    pub fn print(&mut self, text: &str, style: Option<TextStyle>) -> Result<()> {
        let text_style = MonoTextStyle::new(
            &FONT_6X10,
            style
                .map(|s| rgba5551_to_eg_rgb888(s.color))
                .unwrap_or(Rgb888::BLACK),
        );

        let textbox_style = TextBoxStyleBuilder::new()
            .height_mode(HeightMode::Exact(VerticalOverdraw::Hidden))
            .build();

        let text_width = WIDTH - 2 * MARGIN;

        // When the text overflows, clear the screen and go back to the top

        let mut remaining = text;

        // TODO messy, clean up

        while !remaining.is_empty() {
            if self.text_cursor_y >= TEXT_BOTTOM {
                self.fill(WHITE);
                self.text_cursor_y = MARGIN;
            }

            let available_height = TEXT_BOTTOM.saturating_sub(self.text_cursor_y);
            let bounds = Rectangle::new(
                Point::new(MARGIN as i32, self.text_cursor_y as i32),
                Size::new(text_width, available_height),
            );

            let chunk = remaining;
            let text_box = TextBox::with_textbox_style(chunk, bounds, text_style, textbox_style);

            remaining = text_box
                .draw(self)
                .map_err(|e| anyhow!("failed to print text in framebuffer: {e}"))?;

            if remaining.is_empty() {
                self.text_cursor_y +=
                    textbox_style.measure_text_height(&text_style, chunk, text_width);
            } else {
                self.text_cursor_y = TEXT_BOTTOM;
            }
        }

        Ok(())
    }

    pub fn progress(&mut self, done: u32, total: u32) -> Result<()> {
        let color = rgba5551_to_eg_rgb888(SUCCESS);

        // Use fixed-point arithmetic in case emulators have not implemented floating point arithmetics
        let progress = (done << 16) / total;
        let width = (WIDTH * progress) >> 16;

        Rectangle::new(
            Point::new(0, HEIGHT.saturating_sub(PROGRESS_HEIGHT) as i32),
            Size::new(width, PROGRESS_HEIGHT),
        )
        .into_styled(PrimitiveStyleBuilder::new().fill_color(color).build())
        .draw(self)
        .map_err(|e| anyhow!("failed to draw frame in framebuffer: {e}"))?;

        Ok(())
    }

    fn uncached_buffer_ptr(&self) -> *mut u16 {
        // The framebuffer must be accessed via the uncached segment to avoid caching glitches

        (n64_specs::map::Segment::KSEG1 as u32 | self.buffer.as_ptr() as u32) as *mut u16
    }
}

pub struct TextStyle {
    pub color: RGBA5551,
}

impl TextStyle {
    pub fn with_color(color: RGBA5551) -> Self {
        Self { color }
    }
}

impl OriginDimensions for Display {
    fn size(&self) -> Size {
        Size::new(WIDTH, HEIGHT)
    }
}

impl DrawTarget for Display {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let buffer_uncached = self.uncached_buffer_ptr();

        for Pixel(coord, color) in pixels.into_iter() {
            if let Ok((x @ 0..=WIDTH, y @ 0..=HEIGHT)) = coord.try_into() {
                let offset: u32 = x + y * WIDTH;

                unsafe {
                    buffer_uncached.add(offset as usize).write_volatile(
                        RGBA5551::from_rgba(color.r(), color.g(), color.b(), 0xFF).raw_value(),
                    );
                }
            }
        }

        Ok(())
    }
}

fn rgba5551_to_eg_rgb888(rgba5551: RGBA5551) -> Rgb888 {
    let rgb8888: RGBA8888 = rgba5551.into();

    Rgb888::new(rgb8888.red(), rgb8888.green(), rgb8888.blue())
}
