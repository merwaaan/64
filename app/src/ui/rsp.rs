use n64_core::rsp::Register;
use egui::{Context, Window};
use strum::IntoEnumIterator;

use crate::{
    emu::{command::Command, event::Event},
    ui::{Widget, reg32},
};

#[derive(Default)]
pub struct RspWidget {
    last_update: Option<[u32; 8]>,
}

impl Widget for RspWidget {
    fn update(&mut self, _ctx: &Context, event: &Event) {
        if let Event::RspUpdate(rsp) = event {
            self.last_update = Some(*rsp);
        }
    }

    fn show(&mut self, ctx: &Context) -> Vec<Command> {
        Window::new("RSP")
            .default_pos([1300.0, 500.0])
            .show(ctx, |ui| {
                if let Some(rsp_regs) = &self.last_update {
                    let mut show_reg = |reg: Register| {
                        reg32(ui, format!("{:>10}", reg), rsp_regs[reg as usize]);
                    };

                    Register::iter().for_each(|reg| {
                        show_reg(reg);
                    });
                }
            });

        vec![]
    }
}
