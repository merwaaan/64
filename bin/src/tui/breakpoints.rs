use n64::breakpoints::Breakpoint;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::symbols::border;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Padding, Paragraph, Widget};

use n64::cpu::CPU;

pub struct BreakpointsWidget<'a> {
    pub cpu: &'a CPU,
}

impl Widget for BreakpointsWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let block = Block::bordered()
            .title(" Breakpoints ".bold())
            .border_set(border::THICK)
            .padding(Padding::uniform(1));

        let inner = block.inner(area);

        block.render(area, buffer);

        let lines: Vec<Line> = self
            .cpu
            .breakpoints
            .breakpoints
            .iter()
            .map(|breakpoint| match breakpoint {
                Breakpoint::Address(address) => Line::from(format!("{:08X}", address)),
            })
            .collect();

        Paragraph::new(Text::from(lines)).render(inner, buffer);
    }
}
