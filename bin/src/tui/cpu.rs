use n64::registers::Registers;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Widget};

use n64::cpu::CPU;

pub struct CpuWidget<'a> {
    pub cpu: &'a CPU,
}

impl Widget for CpuWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(" CPU ".bold())
            .border_set(border::THICK);
        let inner = block.inner(area);
        block.render(area, buf);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(16),
                Constraint::Length(1),
            ])
            .split(inner);

        let pc_line = Line::from(format!(" PC   {:016x}", self.cpu.regs.pc)).yellow();
        Paragraph::new(pc_line).render(chunks[0], buf);

        let reg_panels = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        let left_lines: Vec<Line> = (0..16)
            .map(|i| {
                let v = self.cpu.regs.gpr[i];
                Line::from(format!("r{:2} {:016x} {}", i, v, Registers::gpr_name(i)))
            })
            .collect();

        let right_lines: Vec<Line> = (16..32)
            .map(|i| {
                let v = self.cpu.regs.gpr[i];
                Line::from(format!("r{:2} {:016x} {}", i, v, Registers::gpr_name(i)))
            })
            .collect();

        Paragraph::new(left_lines).render(reg_panels[0], buf);
        Paragraph::new(right_lines).render(reg_panels[1], buf);

        // let hi_lo = Line::from(format!(
        //     " HI   {:016x}   LO   {:016x}",
        //     self.cpu.regs.hi, self.cpu.regs.lo
        // ))
        // .cyan();
        // Paragraph::new(hi_lo).render(chunks[2], buf);
    }
}
