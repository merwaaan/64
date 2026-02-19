use std::path::PathBuf;

use egui::{Context, Key, MenuBar, TopBottomPanel, UiKind};

use crate::{
    emu::{command::Command, core_thread::CoreThread, event::Event},
    ui::{
        ai::AiWidget,
        breakpoints::BreakpointsWidget,
        colors::Color,
        framebuffer::FramebufferWidget,
        instructions::{InstructionsSettings, InstructionsWidget},
        memory::{MemorySettings, MemoryWidget},
        mi::MiWidget,
        registers::RegistersWidget,
        rsp::RspWidget,
        si::SiWidget,
        text::Text,
        vi::ViWidget,
    },
};

pub mod ai;
pub mod breakpoints;
pub mod colors;
pub mod framebuffer;
pub mod instructions;
pub mod memory;
pub mod mi;
pub mod registers;
pub mod rsp;
pub mod si;
pub mod text;
pub mod vi;

/// Main UI state
///
/// Widget are displayed if their settings are set
#[derive(Clone, Copy)]
pub enum SettingUpdate {
    Instructions(Option<InstructionsSettings>),
    Registers(Option<()>),
    Memory(Option<MemorySettings>),
    Framebuffer(Option<()>),
    // TODO others?
}

pub trait Widget {
    fn init(&mut self) -> Vec<Command> {
        vec![]
    }

    /// Update the widget in response to events from the core thread
    fn update(&mut self, ctx: &Context, event: &Event);

    /// Show the widget in the UI and produce commands to send to the core thread
    fn show(&mut self, ctx: &Context) -> Vec<Command>;
}

#[derive(Debug, Clone, Copy)]
pub enum Status {
    Running,
    Paused,
    Panicked,
}

pub struct Ui {
    widgets: Vec<Box<dyn Widget>>,

    core_thread: CoreThread,
    status: Status,

    // Last ROM loaded, for restarting after a panic
    last_rom_path: Option<PathBuf>,
}

impl Ui {
    pub fn new() -> Self {
        let core_thread = CoreThread::new();

        Self {
            widgets: vec![
                Box::new(InstructionsWidget::default()),
                Box::new(MemoryWidget::default()),
                Box::new(RegistersWidget::default()),
                Box::new(MiWidget::default()),
                Box::new(ViWidget::default()),
                Box::new(SiWidget::default()),
                Box::new(AiWidget::default()),
                Box::new(RspWidget::default()),
                Box::new(FramebufferWidget::default()),
                Box::new(BreakpointsWidget::default()),
            ],
            core_thread,
            status: Status::Paused,
            last_rom_path: None,
        }
    }

    pub fn load_rom(&mut self, path: PathBuf) {
        for widget in self.widgets.iter_mut() {
            let widget_commands = widget.init();

            for command in widget_commands {
                self.core_thread.send_command(command);
            }
        }

        self.core_thread
            .send_command(Command::LoadRom(path.clone()));

        self.last_rom_path = Some(path);
    }

    fn poll_core_thread(&mut self, ctx: &Context) {
        while let Some(event) = self.core_thread.poll_event() {
            match &event {
                Event::StatusUpdate(status) => self.status = *status,
                _ => {}
            }

            for widget in self.widgets.iter_mut() {
                widget.update(ctx, &event);
            }
        }
    }
}

impl eframe::App for Ui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_core_thread(ctx);

        // Handle inputs

        ctx.input(|input| {
            if matches!(self.status, Status::Running) && input.key_pressed(Key::Enter) {
                self.core_thread.send_command(Command::Pause);
            }

            if matches!(self.status, Status::Paused) {
                if input.key_pressed(Key::Enter) {
                    self.core_thread.send_command(Command::Resume);
                }

                if input.key_pressed(Key::Space) {
                    self.core_thread.send_command(Command::Step);
                }
            }

            if input.key_pressed(Key::Escape) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });

        // Render widgets

        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                // Load ROM
                if ui.button("Load ROM…").clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("ROM files", &["n64", "z64", "v64", "zip"])
                        .pick_file()
                {
                    self.load_rom(path);

                    ui.close_kind(UiKind::Menu);
                }

                // Restart

                if let Some(last_rom_path) = &self.last_rom_path
                    && ui.button("↻ Restart").clicked()
                {
                    self.load_rom(last_rom_path.clone());

                    self.status = Status::Paused;
                }

                // Pause/Resume/Step

                match self.status {
                    Status::Running => {
                        if ui.button("⏸ Pause").clicked() {
                            self.core_thread.send_command(Command::Pause);
                        }
                    }
                    Status::Paused => {
                        if ui.button("▶ Resume").clicked() {
                            self.core_thread.send_command(Command::Resume);
                        }
                    }
                    _ => {}
                }

                if ui.button("⏭ Step").clicked() {
                    self.core_thread.send_command(Command::Step);
                }

                // panicked :(

                if matches!(self.status, Status::Panicked) {
                    Text::new("⚠ Core panicked").color(Color::Error).show(ui);
                }
            });
        });

        for widget in self.widgets.iter_mut() {
            let commands = widget.show(ctx);

            for command in commands {
                self.core_thread.send_command(command);
            }
        }

        ctx.request_repaint();
    }
}

pub fn parse_hex(s: &str) -> Option<u64> {
    let s = s.trim().trim_start_matches("0x").trim_start_matches("0X");

    if s.is_empty() {
        return None;
    }

    u64::from_str_radix(s, 16).ok()
}

pub fn reg32(ui: &mut egui::Ui, name: impl AsRef<str>, value: u32) {
    ui.horizontal(|ui| {
        Text::new(name).color(Color::Light).show(ui);
        Text::new(format!("{:08X}", value)).show(ui);
    });
}

pub fn reg64(ui: &mut egui::Ui, name: impl AsRef<str>, value: u64) {
    ui.horizontal(|ui| {
        Text::new(name).color(Color::Light).show(ui);
        Text::new(format!("{:08X} {:08X}", (value >> 32) as u32, value as u32)).show(ui);
    });
}
