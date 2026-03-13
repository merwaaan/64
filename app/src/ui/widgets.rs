use std::{
    collections::HashSet,
    sync::atomic::{AtomicU32, Ordering},
};

use egui::{CollapsingHeader, Window};

use crate::{command::Command, event::Event, ui::Data};

pub mod ai_widget;
pub mod controller_widget;
pub mod cop0_widget;
pub mod cop1_widget;
pub mod cpu_widget;
pub mod dp_widget;
pub mod events_widget;
pub mod framebuffer_widget;
pub mod isviewer_widget;
pub mod memory_widget;
pub mod mi_widget;
pub mod pi_widget;
pub mod si_widget;
pub mod sp_widget;
pub mod tlb_widget;
pub mod vi_widget;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct WidgetId(u32);

impl Default for WidgetId {
    fn default() -> Self {
        static NEXT_WIDGET_ID: AtomicU32 = AtomicU32::new(1);

        Self(NEXT_WIDGET_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// Base widget trait.
pub trait Widget {
    /// Get the unique identifier for the widget
    fn id(&self) -> WidgetId;

    /// Get the data that the widget needs from the core thread
    fn requested_data(&mut self, only_if_changed: bool) -> Option<HashSet<Data>>;

    /// Update the widget in response to events from the core thread
    fn update(&mut self, context: &egui::Context, event: &Event);
}

/// Widget at the root of the UI, associated with the top-level context.
pub trait RootWidget: Widget {
    /// Show the widget in the UI and produce commands to send to the core thread
    fn show(&mut self, ctx: &egui::Context) -> Vec<Command>;

    fn closed(&self) -> bool;
}

/// Widget that's contained in some oher UI component, associated with a specific UI context.
pub trait ChildWidget: Widget {
    /// Show the widget in the UI and produce commands to send to the core thread
    fn show(&mut self, ui: &mut egui::Ui) -> Vec<Command>;
}

/// Wraps a widget in a collapsing header.
/// Sends commands to request the data needed for the widget when it's expanded.
pub struct Collapsing<W: ChildWidget> {
    id: WidgetId,
    title: String,
    open: bool,
    open_on_last_request: Option<bool>,
    widget: W,
}

impl<W: ChildWidget + Default> Collapsing<W> {
    pub fn new(title: &str, open: bool) -> Self {
        Self {
            id: WidgetId::default(),
            title: title.to_string(),
            open,
            open_on_last_request: None,
            widget: W::default(),
        }
    }
}

impl<W: ChildWidget> ChildWidget for Collapsing<W> {
    fn show(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        let mut commands = vec![];

        let response = CollapsingHeader::new(&self.title)
            .open(Some(self.open))
            .show(ui, |ui| {
                commands.extend(self.widget.show(ui));
            });

        if response.header_response.clicked() {
            self.open = !self.open;
        }

        commands
    }
}

impl<W: ChildWidget> Widget for Collapsing<W> {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn requested_data(&mut self, only_if_changed: bool) -> Option<HashSet<Data>> {
        if only_if_changed {
            if self.open_on_last_request != Some(self.open) {
                self.open_on_last_request = Some(self.open);

                if self.open {
                    self.widget.requested_data(false)
                } else {
                    Some(HashSet::new())
                }
            } else {
                None
            }
        } else {
            if self.open {
                self.widget.requested_data(false)
            } else {
                Some(HashSet::new())
            }
        }
    }

    fn update(&mut self, context: &egui::Context, event: &Event) {
        self.widget.update(context, event);
    }
}

/// Wraps a widget in a floating window.
/// Sends commands to request the data needed for the widget when it's created and closed.
pub struct Floating<W: ChildWidget> {
    id: WidgetId,
    title: String,
    widget: W,
    open: bool,
}

impl<W: ChildWidget + Default> Floating<W> {
    pub fn new(title: &str) -> Self {
        Self {
            id: WidgetId::default(),
            title: title.to_string(),
            widget: W::default(),
            open: true,
        }
    }
}

impl<W: ChildWidget> RootWidget for Floating<W> {
    fn show(&mut self, ctx: &egui::Context) -> Vec<Command> {
        let mut commands = vec![];

        Window::new("Memory")
            .default_pos([800.0, 1000.0])
            .open(&mut self.open)
            .show(ctx, |ui| {
                commands.extend(self.widget.show(ui));
            });

        commands
    }

    fn closed(&self) -> bool {
        !self.open
    }
}

impl<W: ChildWidget> Widget for Floating<W> {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn requested_data(&mut self, only_if_changed: bool) -> Option<HashSet<Data>> {
        self.widget.requested_data(only_if_changed)
    }

    fn update(&mut self, context: &egui::Context, event: &Event) {
        self.widget.update(context, event);
    }
}
