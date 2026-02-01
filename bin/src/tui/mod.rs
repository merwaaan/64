mod breakpoints;
mod cpu;
mod instructions;
pub mod logger;

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

use n64::{cart::Cart, cpu::CPU};

use crate::tui::breakpoints::BreakpointsWidget;

use self::cpu::CpuWidget;
use self::instructions::InstructionsWidget;

#[derive(Debug)]
pub enum State {
    Running,
    Paused,
    Exited,
}

// TODO move app out of TUI
pub struct App {
    pub state: State,
    pub cpu: CPU,
    pub cart: Cart,
    pub logs: Vec<String>,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !matches!(self.state, State::Exited) {
            self.logs.extend(logger::drain_logs()); // TODO cleaner way to do this?

            for _ in 0..100 {
                if !matches!(self.state, State::Running) {
                    break;
                }

                let hit_breakpoint = self.cpu.step();

                if hit_breakpoint {
                    self.state = State::Paused;
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
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Release {
                    match key_event.code {
                        KeyCode::Esc => self.state = State::Exited,
                        KeyCode::Enter => {
                            if matches!(self.state, State::Paused) {
                                self.state = State::Running;
                            }
                        }
                        KeyCode::Char(' ') => {
                            self.cpu.step();
                        }
                        _ => {}
                    }
                }
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
                Constraint::Max(25),
            ])
            .split(main_and_log[0]);

        InstructionsWidget { cpu: &self.cpu }.render(panels[0], buf);
        CpuWidget { cpu: &self.cpu }.render(panels[1], buf);
        BreakpointsWidget { cpu: &self.cpu }.render(panels[2], buf);

        // Logs
        // TODO move to widget

        let log_block = Block::bordered()
            .title(Line::from(" Log ".bold()))
            .border_set(border::THICK);

        let log_lines: Vec<Line> = self.logs.iter().map(|s| Line::from(s.as_str())).collect();

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
