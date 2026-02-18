use n64::vi::{Register, Vi};

use crate::emu::event::Event;

#[derive(Default)]
pub struct ViWidget {
    last_update: Option<Vi>,
}

impl ViWidget {
    pub fn update(&mut self, event: &Event) {
        if let Event::ViUpdate(vi) = event {
            self.last_update = Some(*vi);
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        if let Some(mi) = &self.last_update {
            let mut reg = |reg: Register| {
                ui.horizontal(|ui| {
                    ui.monospace(format!("{:>11?}", reg));
                    ui.monospace(format!("{:08X}", mi.regs[reg as usize]));
                });
            };

            reg(Register::Status);
            reg(Register::FramebufferAddr);
            reg(Register::Width);
            reg(Register::InterruptScanline);
            reg(Register::CurrentScanline);
            reg(Register::Burst);
            reg(Register::VSync);
            reg(Register::HSync);
            reg(Register::HSyncLeap);
            reg(Register::HVideo);
            reg(Register::VVideo);
            reg(Register::VBurst);
            reg(Register::XScale);
            reg(Register::YScale);
        }
    }
}
