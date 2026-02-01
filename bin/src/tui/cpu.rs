use n64::registers::Registers;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, Padding, Paragraph, Widget};

use n64::cpu::CPU;

pub struct CpuWidget<'a> {
    pub cpu: &'a CPU,
}

impl Widget for CpuWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(" CPU ".bold())
            .border_set(border::THICK)
            .padding(Padding::uniform(1));

        let inner = block.inner(area);

        block.render(area, buf);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(16),
                Constraint::Length(1),
            ])
            .split(inner);

        let pc_line = Line::from(format!("PC {:016X}", self.cpu.regs.pc)).yellow();
        Paragraph::new(pc_line).render(chunks[0], buf);

        let mult_line = Line::from(format!(
            "HI/LO {:08X} {:08X}",
            self.cpu.regs.mult_hi, self.cpu.regs.mult_lo
        ))
        .cyan();
        Paragraph::new(mult_line).render(chunks[1], buf);

        let reg_panels = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[2]);

        let left_lines: Vec<Line> = (0..16)
            .map(|i| {
                Line::from(format!(
                    "{:2} {:08X} {:08X}",
                    Registers::gpr_name(i),
                    self.cpu.regs.gpr[i] >> 32,
                    self.cpu.regs.gpr[i] & 0xFFFFFFFF
                ))
            })
            .collect();

        let right_lines: Vec<Line> = (16..32)
            .map(|i| {
                Line::from(format!(
                    "{:2} {:08X} {:08X}",
                    Registers::gpr_name(i),
                    self.cpu.regs.gpr[i] >> 32,
                    self.cpu.regs.gpr[i] & 0xFFFFFFFF
                ))
            })
            .collect();

        Paragraph::new(left_lines).render(reg_panels[0], buf);
        Paragraph::new(right_lines).render(reg_panels[1], buf);

        // let hi_lo = Line::from(format!(
        //     " HI   {:016X}   LO   {:016X}",
        //     self.cpu.regs.hi, self.cpu.regs.lo
        // ))
        // .cyan();
        // Paragraph::new(hi_lo).render(chunks[2], buf);
    }
}
