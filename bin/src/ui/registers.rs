use iced::{
    Color, Element, Font, Theme,
    font::{Family, Weight},
    widget::{Text, column, row, text},
};
use n64::registers::Registers;

use crate::{emu::event::Event, ui::UiEvent};

#[derive(Default)]
pub struct RegistersWidget {
    pub cpu_regs: Registers,
}

impl RegistersWidget {
    pub fn update(&mut self, event: &Event) {
        match event {
            Event::Update { cpu_regs, .. } => {
                self.cpu_regs = cpu_regs.clone();
            }
        }
    }

    pub fn view(&self) -> Element<'_, UiEvent> {
        let mut regs_row = row![];

        for col in 0..2 {
            let mut regs_col = column![];

            for row in 0..16 {
                let reg_index = col * 16 + row;

                let name = Registers::gpr_name(reg_index);
                let value = self.cpu_regs.gpr[reg_index].get64();

                regs_col = regs_col.push(
                    row![
                        text(name)
                            .style(|_theme: &Theme| text::Style {
                                color: Some(Color::from_rgb(0.8, 0.8, 1.0)),
                                ..Default::default()
                            })
                            .font(Font {
                                family: Family::Monospace,
                                weight: Weight::Bold,
                                ..Default::default()
                            }),
                        text(format!("{:08X}", (value >> 32))).font(Font {
                            family: Family::Monospace,
                            ..Default::default()
                        }),
                        text(format!("{:08X}", value as u32)).font(Font {
                            family: Family::Monospace,
                            ..Default::default()
                        })
                    ]
                    .spacing(10),
                );
            }

            regs_row = regs_row.push(regs_col).spacing(20);
        }

        let layout = column![Text::new(format!("PC: {:08X}", self.cpu_regs.pc)), regs_row];

        layout.spacing(2).into()
    }
}
