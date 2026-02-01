use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::symbols::border;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Padding, Paragraph, Widget};

use n64::cpu::CPU;

pub struct MemoryWidget<'a> {
    pub cpu: &'a CPU,
}

impl Widget for MemoryWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let block = Block::bordered()
            .title(" Memory ".bold())
            .border_set(border::THICK)
            .padding(Padding::uniform(1));

        let inner = block.inner(area);

        block.render(area, buffer);

        let lines: Vec<Line> = (0..32usize)
            .map(|offset| {
                let address = 0x8024_1800 + offset * 16;

                let v0 = self.cpu.read(address as u32);
                let v1 = self.cpu.read((address + 4) as u32);
                let v2 = self.cpu.read((address + 8) as u32);
                let v3 = self.cpu.read((address + 12) as u32);

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
