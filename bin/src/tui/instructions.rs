use n64::instructions::{Opcode, decode};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Padding, Paragraph, Widget};

use n64::system::System;

pub struct InstructionsWidget<'a> {
    pub system: &'a System,
}

impl Widget for InstructionsWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let block = Block::bordered()
            .title(" Instructions ".bold())
            .border_set(border::THICK)
            .padding(Padding::uniform(1));

        let inner = block.inner(area);

        block.render(area, buffer);

        let lines: Vec<Line> = (self.system.cpu.regs.pc..self.system.cpu.regs.pc + 16 * 4)
            .step_by(4)
            .map(|addr| {
                let instruction = self.system.read(addr);

                let opcode = Opcode(instruction);
                let handler = decode(opcode);
                let disassembly = handler.disassemble(self.system, opcode);

                let addr_style = if addr == self.system.cpu.regs.pc {
                    Style::default().blue().reversed()
                } else {
                    Style::default().blue()
                };

                let mut spans = vec![
                    Span::styled(format!("{:08X}:", addr), addr_style),
                    Span::styled(
                        format!(" {}", disassembly.mnemonics),
                        Style::default().white(),
                    ),
                ];

                if let Some(hint) = disassembly.hint {
                    spans.push(Span::styled(
                        format!("   // {}", hint),
                        Style::default().dim(),
                    ))
                }

                Line::from(spans)
            })
            .collect();

        Paragraph::new(Text::from(lines)).render(inner, buffer);
    }
}
