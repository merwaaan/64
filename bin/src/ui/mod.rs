use std::path::PathBuf;

use egui::{Context, Key, MenuBar, TopBottomPanel, UiKind};

use crate::{
    Args,
    emu::{command::Command, event::Event, runner::Runner},
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
/// Widget are displayed when their settings have a value
#[derive(Clone, Copy)]
pub enum SettingUpdate {
    Instructions(Option<InstructionsSettings>),
    Registers(Option<()>),
    Memory(Option<MemorySettings>),
    Framebuffer(Option<()>),
}

pub trait Widget {
    fn init(&mut self) -> Vec<Command> {
        vec![]
    }

    fn update(&mut self, ctx: &Context, event: &Event);

    fn show(&mut self, ctx: &Context) -> Vec<Command>;
}

pub struct Ui {
    runner: Option<Runner>,

    paused: bool,

    widgets: Vec<Box<dyn Widget>>,
}

impl Ui {
    pub fn new(args: &Args) -> Self {
        // Setup the widgets

        let mut widgets: Vec<Box<dyn Widget>> = vec![
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
        ];

        // Initialize the runner

        let runner = Runner::new();

        // Send the initialstartup commands

        for widget in widgets.iter_mut() {
            let commands = widget.init();

            for command in commands {
                runner.send_command(command);
            }
        }

        // Start paused or not

        let paused = true;

        if paused {
            runner.send_command(if paused {
                Command::Pause
            } else {
                Command::Resume
            });
        }

        // Load the ROM

        if let Some(path) = &args.rom {
            runner.send_command(Command::LoadRom(PathBuf::from(path)));
        }

        Self {
            runner: Some(runner),
            paused,
            widgets,
        }
    }

    fn poll_runner(&mut self, ctx: &Context) {
        while let Some(event) = self.runner.as_ref().and_then(|r| r.poll_event()) {
            match event {
                Event::Pause => {
                    self.paused = true;
                }
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
        self.poll_runner(ctx);

        // Handle events

        ctx.input(|input| {
            if let Some(runner) = self.runner.as_ref() {
                if input.key_pressed(Key::Enter) {
                    if self.paused {
                        runner.send_command(Command::Resume);
                    } else {
                        runner.send_command(Command::Pause);
                    }

                    self.paused = !self.paused;
                }

                if self.paused && input.key_pressed(Key::Space) {
                    runner.send_command(Command::Step);
                }
            }

            // TODO clean exit
            if input.key_pressed(Key::Escape) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });

        // Render widgets

        //TODO to widget
        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                if let Some(runner) = &self.runner
                    && ui.button("Load ROM…").clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("ROM files", &["n64", "z64", "v64", "zip"])
                        .pick_file()
                {
                    runner.send_command(Command::LoadRom(path));
                    ui.close_kind(UiKind::Menu);
                }

                if let Some(runner) = &self.runner
                    && ui
                        .button(if self.paused {
                            "▶ Resume"
                        } else {
                            "⏸ Pause"
                        })
                        .clicked()
                {
                    if self.paused {
                        runner.send_command(Command::Resume);
                    } else {
                        runner.send_command(Command::Pause);
                    }

                    self.paused = !self.paused;
                }

                if self.paused
                    && let Some(runner) = &self.runner
                    && ui.button("⏭ Step").clicked()
                {
                    runner.send_command(Command::Step);
                }
            });
        });

        for widget in self.widgets.iter_mut() {
            let commands = widget.show(ctx);

            if let Some(runner) = &self.runner {
                for command in commands {
                    runner.send_command(command);
                }
            }
        }

        ctx.request_repaint(); // TODO???
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
