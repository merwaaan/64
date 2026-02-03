use n64::events::Events;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::symbols::border;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Padding, Paragraph, Widget};

pub struct EventsWidget<'a> {
    pub events: &'a Events,
}

impl Widget for EventsWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let block = Block::bordered()
            .title(" Events ".bold())
            .border_set(border::THICK)
            .padding(Padding::uniform(1));

        let inner = block.inner(area);

        block.render(area, buffer);

        // TODO avoid copy
        let lines: Vec<Line> = self
            .events
            .events
            .iter()
            .copied()
            .map(|event| Line::from(format!("{:?}", event)))
            .collect();

        Paragraph::new(Text::from(lines)).render(inner, buffer);
    }
}
