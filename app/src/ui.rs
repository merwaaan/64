use std::path::PathBuf;

use egui::{
    CentralPanel, Context, CursorIcon, Key, MenuBar, ScrollArea, SidePanel, TopBottomPanel, UiKind,
};
use gilrs::Gilrs;
use n64_core::controller::Button;

use crate::{
    command::{Command, ControllerInput},
    core_thread::CoreThread,
    event::Event,
    ui::{
        text::Text,
        widgets::{
            ChildWidget, Collapsing, Floating, RootWidget, Widget,
            ai_widget::AiWidget,
            controller_widget::ControllerWidget,
            cop0_widget::Cop0Widget,
            cop1_widget::Cop1Widget,
            cpu_widget::CpuWidget,
            dp_widget::DpWidget,
            events_widget::EventsWidget,
            framebuffer_widget::FramebufferWidget,
            isviewer_widget::IsViewerWidget,
            memory_widget::{MemorySettings, MemoryWidget},
            mi_widget::MiWidget,
            pi_widget::PiWidget,
            si_widget::SiWidget,
            sp_widget::SpWidget,
            tlb_widget::TlbWidget,
            vi_widget::ViWidget,
        },
    },
};

pub mod colors;
pub mod text;
pub mod widgets;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Data {
    Memory(MemorySettings),
    Cpu,
    Cop0,
    Cop1,
    Sp,
    Ai,
    Mi,
    Vi,
    Pi,
    Si,
    Dp,
    Tlb,
    IsViewer,
    Events,
    Framebuffer,
    Breakpoints,
}

#[derive(Debug, Clone)]
pub enum Status {
    Running,
    Paused,
    Panicked(String),
}

/// Main UI state
pub struct Ui {
    core_thread: CoreThread,
    status: Status,

    left_widgets: Vec<Box<dyn ChildWidget>>,
    right_widgets: Vec<Box<dyn ChildWidget>>,
    floating_widgets: Vec<Box<dyn RootWidget>>,
    center_widget: Box<dyn ChildWidget>,

    // Last ROM loaded, for restarting after a panic
    last_rom_path: Option<PathBuf>,

    controllers_api: Gilrs,
}

impl Ui {
    pub fn new() -> Self {
        let core_thread = CoreThread::new();

        Self {
            left_widgets: vec![
                Box::new(Collapsing::<CpuWidget>::new("CPU", true)),
                Box::new(Collapsing::<Cop0Widget>::new("COP0", false)),
                Box::new(Collapsing::<Cop1Widget>::new("COP1", false)),
                Box::new(Collapsing::<SpWidget>::new("SP", true)),
            ],
            right_widgets: vec![
                Box::new(Collapsing::<MiWidget>::new("MIPS interface", false)),
                Box::new(Collapsing::<ViWidget>::new("Video interface", false)),
                Box::new(Collapsing::<PiWidget>::new("Peripheral interface", false)),
                Box::new(Collapsing::<AiWidget>::new("Audio interface", false)),
                Box::new(Collapsing::<SiWidget>::new("Serial interface", false)),
                Box::new(Collapsing::<DpWidget>::new("Display processor", false)),
                Box::new(Collapsing::<TlbWidget>::new(
                    "Translation lookaside buffer",
                    false,
                )),
                Box::new(Collapsing::<ControllerWidget>::new("Controller", false)),
                Box::new(Collapsing::<EventsWidget>::new("Events", false)),
                Box::new(Collapsing::<IsViewerWidget>::new("IS Viewer", false)),
            ],
            center_widget: Box::new(FramebufferWidget::default()),
            floating_widgets: vec![Box::new(Floating::<MemoryWidget>::new("Memory"))],

            core_thread,
            status: Status::Paused,

            last_rom_path: None,

            controllers_api: Gilrs::new().unwrap(),
        }
    }

    pub fn load_rom(&mut self, path: PathBuf) {
        self.core_thread
            .send_command(Command::LoadRom(path.clone()));

        self.last_rom_path = Some(path);
    }

    fn poll_core_thread(&mut self, ctx: &Context) {
        while let Some(event) = self.core_thread.poll_event() {
            if let Event::Status(status) = &event {
                self.status = status.clone();
            }

            for widget in self
                .left_widgets
                .iter_mut()
                .chain(self.right_widgets.iter_mut())
            {
                widget.update(ctx, &event);
            }

            for widget in self.floating_widgets.iter_mut() {
                widget.update(ctx, &event);
            }

            self.center_widget.update(ctx, &event);
        }
    }
}

impl eframe::App for Ui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_core_thread(ctx);

        // Handle inputs

        ctx.input(|input| {
            // Keyboard

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

            // Dropped files

            if let Some(file) = input.raw.dropped_files.first()
                && let Some(path) = &file.path
            {
                self.core_thread
                    .send_command(Command::LoadRom(path.clone()));

                self.last_rom_path = Some(path.clone());
            }
        });

        // while let Some(event) = self.controllers_api.next_event() {
        //     if let Some(input) = gilrs_event_to_controller_input(event.event) {
        //         self.core_thread
        //             .send_command(Command::ControllerInput(input));
        //     }
        // }

        // Render widgets

        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                // Load ROM
                if ui.button("Load ROM…").clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("ROM files", n64_core::get_supported_file_extensions())
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

                if ui
                    .add_enabled(
                        matches!(self.status, Status::Paused | Status::Panicked(_)),
                        egui::Button::new("⏭ Step"),
                    )
                    .clicked()
                {
                    self.core_thread.send_command(Command::Step);
                }

                // panicked :(

                if let Status::Panicked(error) = &self.status {
                    Text::new("⚠ Core panicked")
                        .color(colors::ERROR)
                        .show(ui)
                        .on_hover_text(error)
                        .on_hover_cursor(CursorIcon::Help);
                }

                if let Some(rom_name) = &self.last_rom_path.as_ref().and_then(|p| p.file_name()) {
                    Text::new(format!("({})", rom_name.display()))
                        .color(colors::LIGHT)
                        .show(ui);
                }
            });
        });

        let mut commands = Vec::new();

        SidePanel::left("left")
            .exact_width(800.0)
            .resizable(false)
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    for widget in &mut self.left_widgets {
                        commands.extend(widget.show(ui));

                        commands.extend(
                            widget
                                .requested_data(true)
                                .iter()
                                .map(|data| Command::RequestData(widget.id(), data.clone())),
                        );
                    }
                });
            });

        SidePanel::right("right")
            .exact_width(500.0)
            .resizable(false)
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    for widget in &mut self.right_widgets {
                        commands.extend(widget.show(ui));

                        commands.extend(
                            widget
                                .requested_data(true)
                                .iter()
                                .map(|data| Command::RequestData(widget.id(), data.clone())),
                        );
                    }
                });
            });

        for widget in &mut self.floating_widgets {
            // TODO close

            commands.extend(widget.show(ctx));

            commands.extend(
                widget
                    .requested_data(true)
                    .iter()
                    .map(|data| Command::RequestData(widget.id(), data.clone())),
            );
        }

        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("TEST");
                commands.extend(self.center_widget.show(ui));

                commands.extend(
                    self.center_widget
                        .requested_data(true)
                        .iter()
                        .map(|data| Command::RequestData(self.center_widget.id(), data.clone())),
                );
            });
        });

        for command in commands {
            self.core_thread.send_command(command);
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
        Text::new(name).color(colors::LIGHT).show(ui);
        Text::new(format!("{:08X}", value)).show(ui);
    });
}

pub fn reg64(ui: &mut egui::Ui, name: impl AsRef<str>, value: u64) {
    ui.horizontal(|ui| {
        Text::new(name).color(colors::LIGHT).show(ui);
        Text::new(format!("{:08X} {:08X}", (value >> 32) as u32, value as u32)).show(ui);
    });
}

fn gilrs_event_to_controller_input(event: gilrs::EventType) -> Option<ControllerInput> {
    //log::info!("Gilrs event: {:?}", event);

    match event {
        gilrs::EventType::ButtonPressed(button, _code) => gilrs_button_to_controller_button(button)
            .map(|button| ControllerInput::PressButton(button)),

        gilrs::EventType::ButtonReleased(button, _code) => {
            gilrs_button_to_controller_button(button)
                .map(|button| ControllerInput::ReleaseButton(button))
        }

        // gilrs::EventType::AxisChanged(gilrs::Axis::LeftStickX, value, _code) => {
        //     Command::ControllerInput(ControllerInput::AxisChanged(true, value))
        // }
        _ => None,
    }
}

fn gilrs_button_to_controller_button(button: gilrs::Button) -> Option<Button> {
    match button {
        gilrs::Button::Start => Some(Button::Start),
        gilrs::Button::North => Some(Button::B),
        gilrs::Button::East => Some(Button::A),
        gilrs::Button::South => Some(Button::A),
        gilrs::Button::West => Some(Button::B),
        gilrs::Button::LeftTrigger => Some(Button::LeftTrigger),
        gilrs::Button::RightTrigger => Some(Button::RightTrigger),
        gilrs::Button::DPadUp => Some(Button::DUp),
        gilrs::Button::DPadDown => Some(Button::DDown),
        gilrs::Button::DPadLeft => Some(Button::DLeft),
        gilrs::Button::DPadRight => Some(Button::DRight),
        _ => None,
    }
}
