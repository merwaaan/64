use n64::instructions::decode;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::symbols::border;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Paragraph, Widget};

use n64::cpu::CPU;

pub struct InstructionsWidget<'a> {
    pub cpu: &'a CPU,
}

impl Widget for InstructionsWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let block = Block::bordered()
            .title(" Instructions ".bold())
            .border_set(border::THICK);

        let inner = block.inner(area);

        block.render(area, buffer);

        let lines: Vec<Line> = (self.cpu.regs.pc..self.cpu.regs.pc + 16 * 4)
            .step_by(4)
            .map(|address| {
                let instruction = self.cpu.read(address as u32);

                let disassembly = decode(instruction).disassemble(&self.cpu, instruction);

                let line = Line::from(format!("{:08X}: {}", address, disassembly));

                if address == self.cpu.regs.pc {
                    line.reversed().yellow()
                } else {
                    line
                }
            })
            .collect();

        Paragraph::new(Text::from(lines)).render(inner, buffer);
    }
}
