use n64::system::System;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::symbols::border;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Padding, Paragraph, Widget};

pub struct MemoryWidget<'a> {
    pub system: &'a System,
}

impl Widget for MemoryWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let block = Block::bordered()
            .title(" Memory ".bold())
            .border_set(border::THICK)
            .padding(Padding::uniform(1));

        let inner = block.inner(area);

        block.render(area, buffer);

        let lines: Vec<Line> = (0..16usize)
            .map(|offset| {
                let address = 0x801FB9A0 + offset * 16;

                let v0: u32 = self.system.read(address as u32);
                let v1: u32 = self.system.read((address + 4) as u32);
                let v2: u32 = self.system.read((address + 8) as u32);
                let v3: u32 = self.system.read((address + 12) as u32);

                Line::from(vec![
                    Span::styled(
                        format!("{:08X}: ", address),
                        ratatui::style::Style::default().dim(),
                    ),
                    Span::raw(format!("{:08X} {:08X} {:08X} {:08X}", v0, v1, v2, v3)),
                ])
            })
            .collect();

        Paragraph::new(Text::from(lines)).render(inner, buffer);
    }
}
