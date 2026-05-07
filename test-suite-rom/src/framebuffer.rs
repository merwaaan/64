use alloc::vec::Vec;
use anyhow::{Result, anyhow};
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle, StrokeAlignment},
};
use embedded_text::{
    TextBox,
    style::{HeightMode, TextBoxStyleBuilder},
};
use n64_specs::{color::RGBA8888, vi};

use crate::reg_mut_ptr;

pub const WIDTH: u32 = 320;
pub const HEIGHT: u32 = 240;
pub const PIXELS: usize = WIDTH as usize * HEIGHT as usize;

const MARGIN: u32 = 16;

pub const WHITE: RGBA8888 = RGBA8888::from_rgba(0xFF, 0xFF, 0xFF, 0xFF);
pub const SUCCESS: RGBA8888 = RGBA8888::from_rgba(0x4C, 0xAF, 0x50, 0xFF);
pub const WARNING: RGBA8888 = RGBA8888::from_rgba(0xFF, 0x98, 0x00, 0xFF);
pub const ERROR: RGBA8888 = RGBA8888::from_rgba(0xEF, 0x53, 0x50, 0xFF);

pub struct Framebuffer {
    buffer: Vec<u32>,
    text_cursor_y: u32,
}

impl Framebuffer {
    pub fn new() -> Self {
        let buffer = alloc::vec![WHITE.raw_value(); PIXELS];

        // TODO use specific reg for each write instead of base + offset
        let vi_reg_base = reg_mut_ptr(vi::START);

        // TODO be explicit about values
        unsafe {
            vi_reg_base.add(vi::Control::INDEX).write_volatile(12879);

            vi_reg_base
                .add(vi::Origin::INDEX)
                .write_volatile(buffer.as_ptr() as u32);

            vi_reg_base.add(vi::Width::INDEX).write_volatile(WIDTH);

            vi_reg_base.add(vi::InterruptLine::INDEX).write_volatile(2);

            vi_reg_base
                .add(vi::Burst::INDEX)
                .write_volatile(0x03E5_2239);

            vi_reg_base
                .add(vi::VerticalTotal::INDEX)
                .write_volatile(0x0000_020D);

            vi_reg_base
                .add(vi::HorizontalTotal::INDEX)
                .write_volatile(0x0000_0C15);

            vi_reg_base
                .add(vi::HorizontalTotalLeap::INDEX)
                .write_volatile(0x0C15_0C15);

            vi_reg_base
                .add(vi::HorizontalVideo::INDEX)
                .write_volatile(0x006C_02EC);

            vi_reg_base
                .add(vi::VerticalVideo::INDEX)
                .write_volatile(0x0025_01FF);

            vi_reg_base
                .add(vi::VerticalBurst::INDEX)
                .write_volatile(0x000E_0204);

            vi_reg_base
                .add(vi::HorizontalScale::INDEX)
                .write_volatile((0x100 * WIDTH) / 160);

            vi_reg_base
                .add(vi::VerticalScale::INDEX)
                .write_volatile((0x100 * HEIGHT) / 60);
        }

        Self {
            buffer,
            text_cursor_y: MARGIN,
        }
    }

    pub fn fill(&mut self, color: RGBA8888) {
        let buffer_uncached = self.buffer_ptr();

        unsafe {
            for i in 0..PIXELS {
                buffer_uncached.add(i).write_volatile(color.raw_value());
            }
        }
    }

    pub fn print(&mut self, text: &str, color: Option<RGBA8888>) -> Result<()> {
        let text_color = if let Some(color) = color {
            Rgb888::new(color.red(), color.green(), color.blue())
        } else {
            Rgb888::BLACK
        };

        let text_style = MonoTextStyle::new(&FONT_6X10, text_color);

        let bounds = Rectangle::new(
            Point::new(MARGIN as i32, self.text_cursor_y as i32),
            Size::new(WIDTH - 2 * MARGIN, HEIGHT - 2 * MARGIN),
        );

        let textbox_style = TextBoxStyleBuilder::new()
            .height_mode(HeightMode::FitToText)
            .build();

        let text_box = TextBox::with_textbox_style(text, bounds, text_style, textbox_style);
        text_box
            .draw(self)
            .map_err(|e| anyhow!("failed to print text in framebuffer: {e}"))?;

        self.text_cursor_y += text_box.bounding_box().size.height as u32;

        Ok(())
    }

    pub fn frame(&mut self, success: bool) -> Result<()> {
        let color = if success { SUCCESS } else { ERROR };
        let color = Rgb888::new(color.red(), color.green(), color.blue());

        Rectangle::new(Point::zero(), Size::new(WIDTH, HEIGHT))
            .into_styled(
                PrimitiveStyleBuilder::new()
                    .stroke_color(color)
                    .stroke_width(MARGIN / 2)
                    .stroke_alignment(StrokeAlignment::Inside)
                    .build(),
            )
            .draw(self)
            .map_err(|e| anyhow!("failed to draw frame in framebuffer: {e}"))?;

        Ok(())
    }

    fn buffer_ptr(&self) -> *mut u32 {
        // Must be accessed via the uncached segment to avoid caching glitches

        (n64_specs::map::Segment::KSEG1 as u32 | self.buffer.as_ptr() as u32) as *mut u32
    }
}

impl OriginDimensions for Framebuffer {
    fn size(&self) -> Size {
        Size::new(WIDTH, HEIGHT)
    }
}

impl DrawTarget for Framebuffer {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let buffer_uncached = self.buffer_ptr();

        for Pixel(coord, color) in pixels.into_iter() {
            if let Ok((x @ 0..=WIDTH, y @ 0..=HEIGHT)) = coord.try_into() {
                let offset: u32 = x as u32 + y as u32 * WIDTH;

                unsafe {
                    buffer_uncached.add(offset as usize).write_volatile(
                        RGBA8888::from_rgba(color.r(), color.g(), color.b(), 0xFF).raw_value(),
                    );
                }
            }
        }

        Ok(())
    }
}
