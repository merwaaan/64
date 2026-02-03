use n64::registers::Registers;
use n64::system::System;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, Padding, Paragraph, Widget};

pub struct CpuWidget<'a> {
    pub system: &'a System,
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
            ])
            .split(inner);

        let pc_line = Line::from(format!(
            "PC {:016X}    {} {:08X}",
            self.system.cpu.regs.pc, self.system.cpu.step, self.system.cycles
        ))
        .yellow();

        Paragraph::new(pc_line).render(chunks[0], buf);

        let mult_line = Line::from(format!(
            "HI/LO {:08X} {:08X}",
            self.system.cpu.regs.mult_hi.get(),
            self.system.cpu.regs.mult_lo.get()
        ))
        .cyan();

        Paragraph::new(mult_line).render(chunks[1], buf);

        let reg_panels = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ])
            .split(chunks[2]);

        let left_lines: Vec<Line> = (0..16)
            .map(|i| {
                Line::from(format!(
                    "{:2} {:08X} {:08X}",
                    Registers::gpr_name(i),
                    self.system.cpu.regs.gpr[i].get64() >> 32,
                    self.system.cpu.regs.gpr[i].get()
                ))
            })
            .collect();

        let right_lines: Vec<Line> = (16..32)
            .map(|i| {
                Line::from(format!(
                    "{:2} {:08X} {:08X}",
                    Registers::gpr_name(i),
                    self.system.cpu.regs.gpr[i].get64() >> 32,
                    self.system.cpu.regs.gpr[i].get()
                ))
            })
            .collect();

        let cop0_left: Vec<Line> = (0..16)
            .map(|i| {
                Line::from(format!(
                    "{:8} {:08X} {:08X}",
                    Registers::cop0_name(i),
                    self.system.cpu.regs.cop0[i].get64() >> 32,
                    self.system.cpu.regs.cop0[i].get()
                ))
            })
            .collect();

        let cop0_right: Vec<Line> = (16..32)
            .map(|i| {
                Line::from(format!(
                    "{:8} {:08X} {:08X}",
                    Registers::cop0_name(i),
                    self.system.cpu.regs.cop0[i].get64() >> 32,
                    self.system.cpu.regs.cop0[i].get()
                ))
            })
            .collect();

        Paragraph::new(left_lines).render(reg_panels[0], buf);
        Paragraph::new(right_lines).render(reg_panels[1], buf);
        Paragraph::new(cop0_left).render(reg_panels[2], buf);
        Paragraph::new(cop0_right).render(reg_panels[3], buf);
    }
}
