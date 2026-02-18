use std::path::PathBuf;

use egui::{Context, Key, MenuBar, TopBottomPanel, UiKind, Window};

use crate::{
    Args,
    emu::{command::Command, event::Event, runner::Runner},
    ui::{
        breakpoints::BreakpointsWidget,
        framebuffer::FramebufferWidget,
        instructions::{InstructionsSettings, InstructionsWidget},
        memory::{MemorySettings, MemoryWidget},
        mi::MiWidget,
        registers::RegistersWidget,
        vi::ViWidget,
    },
};

pub mod breakpoints;
pub mod framebuffer;
pub mod instructions;
pub mod memory;
pub mod mi;
pub mod registers;
pub mod vi;

#[derive(Clone, Copy)]
pub enum SettingUpdate {
    Instructions(Option<InstructionsSettings>),
    Registers(Option<()>),
    Memory(Option<MemorySettings>),
    Framebuffer(Option<()>),
}

pub struct Ui {
    runner: Option<Runner>,

    paused: bool,

    // Widgets
    instructions: Option<InstructionsWidget>,
    registers: Option<RegistersWidget>,
    memory: Option<MemoryWidget>,
    mi: Option<MiWidget>,
    vi: Option<ViWidget>,
    framebuffer: Option<FramebufferWidget>,
    breakpoints: Option<BreakpointsWidget>,
}

impl Ui {
    pub fn new(args: &Args) -> Self {
        let runner = Runner::new();

        let instructions_widget = InstructionsWidget::default();
        let memory_widget = MemoryWidget::default();

        // Send the initial settings to the runner

        // TODO just send from inside the widget???
        runner.send_command(Command::SetSetting(SettingUpdate::Instructions(Some(
            instructions_widget.settings,
        ))));

        runner.send_command(Command::SetSetting(SettingUpdate::Memory(Some(
            memory_widget.settings,
        ))));

        runner.send_command(Command::SetSetting(SettingUpdate::Registers(Some(()))));

        runner.send_command(Command::SetSetting(SettingUpdate::Framebuffer(Some(()))));

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
            instructions: Some(instructions_widget),
            registers: Some(RegistersWidget::default()),
            memory: Some(memory_widget),
            mi: Some(MiWidget::default()),
            vi: Some(ViWidget::default()),
            framebuffer: Some(FramebufferWidget::default()),
            breakpoints: Some(BreakpointsWidget::default()),
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

            if let Some(instructions) = self.instructions.as_mut() {
                instructions.update(&event);
            }

            if let Some(registers) = self.registers.as_mut() {
                registers.update(&event);
            }

            if let Some(memory) = self.memory.as_mut() {
                memory.update(&event);
            }

            if let Some(mi) = self.mi.as_mut() {
                mi.update(&event);
            }

            if let Some(vi) = self.vi.as_mut() {
                vi.update(&event);
            }

            if let Some(framebuffer) = self.framebuffer.as_mut() {
                framebuffer.update(ctx, &event);
            }

            if let Some(breakpoints) = self.breakpoints.as_mut() {
                breakpoints.update(&event);
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
                ui.menu_button("File", |ui| {
                    if let Some(runner) = &self.runner
                        && ui.button("Load ROM…").clicked()
                        && let Some(path) = rfd::FileDialog::new()
                            .add_filter("ROM files", &["n64", "z64", "v64"])
                            .pick_file()
                    {
                        runner.send_command(Command::LoadRom(path));
                    }

                    ui.close_kind(UiKind::Menu);
                });

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

        Window::new("Instructions")
            .default_pos([0.0, 100.0])
            .show(ctx, |ui| {
                if let Some(instructions) = self.instructions.as_ref() {
                    instructions.show(ui);
                }
            });

        Window::new("Registers")
            .default_pos([800.0, 100.0])
            .show(ctx, |ui| {
                if let Some(registers) = self.registers.as_ref() {
                    registers.show(ui);
                }
            });

        Window::new("Memory")
            .default_pos([1600.0, 100.0])
            .show(ctx, |ui| {
                if let Some(memory) = self.memory.as_mut() {
                    memory.show(ui, &mut self.runner);
                }
            });

        Window::new("MI")
            .default_pos([400.0, 600.0])
            .show(ctx, |ui| {
                if let Some(mi) = self.mi.as_mut() {
                    mi.show(ui);
                }
            });

        Window::new("VI").default_pos([0.0, 800.0]).show(ctx, |ui| {
            if let Some(vi) = self.vi.as_mut() {
                vi.show(ui);
            }
        });

        Window::new("Framebuffer")
            .default_pos([400.0, 1000.0])
            .show(ctx, |ui| {
                if let Some(framebuffer) = self.framebuffer.as_mut() {
                    framebuffer.show(ui);
                }
            });

        Window::new("Breakpoints")
            .default_pos([900.0, 1000.0])
            .show(ctx, |ui| {
                if let Some(breakpoints) = self.breakpoints.as_mut() {
                    breakpoints.show(ui, &mut self.runner);
                }
            });

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
