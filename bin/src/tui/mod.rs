mod breakpoints;
mod cpu;
mod instructions;
pub mod logger;
mod memory;

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::symbols::border;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Paragraph, Widget};
use ratatui::{DefaultTerminal, Frame};
use std::io::{self};
use std::time::Duration;

use n64::cpu::CPU;

use crate::tui::breakpoints::BreakpointsWidget;
use crate::tui::memory::MemoryWidget;

use self::cpu::CpuWidget;
use self::instructions::InstructionsWidget;

#[derive(Debug)]
pub enum RunMode {
    Loop,
    Steps(usize),
}

#[derive(Debug)]
pub enum State {
    Running(RunMode),
    Paused,
    Exited,
}

// TODO move app out of TUI
pub struct App {
    pub state: State,
    pub cpu: CPU,
    pub logs: Vec<String>,
}

const INSTR_PER_FRAME: usize = 100_000;

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !matches!(self.state, State::Exited) {
            self.logs.extend(logger::drain_logs()); // TODO cleaner way to do this?

            if !matches!(self.state, State::Paused) {
                for _ in 0..INSTR_PER_FRAME {
                    let hit_breakpoint = self.cpu.step();

                    if hit_breakpoint {
                        self.state = State::Paused;
                        break;
                    } else if let State::Running(RunMode::Steps(steps)) = &self.state {
                        let remaining = steps.saturating_sub(1);

                        if remaining == 0 {
                            self.state = State::Paused;
                            break;
                        } else {
                            self.state = State::Running(RunMode::Steps(remaining));
                        }
                    }
                }
            }

            terminal.draw(|frame| self.draw(frame))?;

            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(0))?
            && let Event::Key(key_event) = event::read()?
            && key_event.kind == KeyEventKind::Release
        {
            match key_event.code {
                KeyCode::Esc => self.state = State::Exited,
                KeyCode::Char('p') => {
                    self.state = State::Paused;
                }
                KeyCode::Enter => {
                    self.state = State::Running(RunMode::Loop);
                }
                KeyCode::Char(' ') => {
                    self.state = State::Running(RunMode::Steps(1));
                }
                KeyCode::Char('n') => {
                    self.state = State::Running(RunMode::Steps(100));
                }
                _ => {}
            }
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Root

        let title = Line::from(" N64 ".bold());

        let instructions = Line::from(vec![
            " Step ".into(),
            "<Enter>".blue().bold(),
            " Quit ".into(),
            "<Esc>".blue().bold(),
        ]);

        let root_block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let inner = root_block.inner(area);

        root_block.render(area, buf);

        //

        let main_and_log = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Length(10)])
            .split(inner);

        //

        let panels = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Max(20),
            ])
            .split(main_and_log[0]);

        InstructionsWidget { cpu: &self.cpu }.render(panels[0], buf);
        CpuWidget { cpu: &self.cpu }.render(panels[1], buf);
        MemoryWidget { cpu: &self.cpu }.render(panels[2], buf);
        BreakpointsWidget { cpu: &self.cpu }.render(panels[3], buf);

        // Logs
        // TODO move to widget

        let log_block = Block::bordered()
            .title(Line::from(" Log ".bold()))
            .border_set(border::THICK);

        let log_lines: Vec<Line> = self
            .logs
            .iter()
            .rev()
            .map(|s| Line::from(s.as_str()))
            .collect();

        let log_content = if log_lines.is_empty() {
            Text::from("(no messages)")
        } else {
            Text::from(log_lines)
        };

        Paragraph::new(log_content)
            .block(log_block)
            .render(main_and_log[1], buf);
    }
}
