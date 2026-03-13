use std::collections::HashSet;

use egui::Context;
use n64_core::controller::Button;

use crate::{
    command::{Command, ControllerInput},
    event::Event,
    ui::{
        Data,
        widgets::{ChildWidget, Widget, WidgetId},
    },
};

#[derive(Default)]
pub struct ControllerWidget {
    id: WidgetId,

    a: bool,
    b: bool,
    start: bool,
}

impl Widget for ControllerWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn requested_data(&mut self, _only_if_changed: bool) -> Option<HashSet<Data>> {
        None
    }

    fn update(&mut self, _ctx: &Context, _event: &Event) {}
}

impl ChildWidget for ControllerWidget {
    fn show(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        let mut commands = Vec::new();

        button_checkbox(ui, &mut commands, Button::A, &mut self.a);
        button_checkbox(ui, &mut commands, Button::B, &mut self.b);
        button_checkbox(ui, &mut commands, Button::Start, &mut self.start);

        commands
    }
}

fn button_checkbox(
    ui: &mut egui::Ui,
    commands: &mut Vec<Command>,
    button: Button,
    pressed: &mut bool,
) {
    if ui.checkbox(pressed, format!("{:?}", button)).changed() {
        if *pressed {
            commands.push(Command::ControllerInput(ControllerInput::PressButton(
                button,
            )));
        } else {
            commands.push(Command::ControllerInput(ControllerInput::ReleaseButton(
                button,
            )));
        }
    }
}
