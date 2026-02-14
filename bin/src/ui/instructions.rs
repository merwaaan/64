use iced::{
    Color, Element, Font, Theme,
    font::{Family, Weight},
    widget::{Text, column, row, text},
};
use n64::instructions::Disassembly;

use crate::{emu::event::Event, ui::UiEvent};

#[derive(Clone)]
pub struct InstructionData {
    pub address: u32,
    pub disassembly: Disassembly,
}

#[derive(Default)]
pub struct InstructionsWidget {
    pub instructions: Vec<InstructionData>,
}

impl InstructionsWidget {
    pub fn update(&mut self, event: &Event) {
        match event {
            Event::Update { instructions, .. } => {
                self.instructions = instructions.clone();
            }
        }
    }

    pub fn view(&self) -> Element<'_, UiEvent> {
        let mut col = column![];

        for instruction in &self.instructions {
            col = col.push(
                row![
                    text(format!("{:08X}", instruction.address)).font(Font {
                        family: Family::Monospace,
                        weight: Weight::Bold,
                        ..Default::default()
                    }),
                    text(format!(" {}", instruction.disassembly.mnemonics)).font(Font {
                        family: Family::Monospace,
                        ..Default::default()
                    }),
                ]
                .spacing(4), //TODO hint
            );
        }
        // for (chunk_index, chunk) in self.memory.chunks(16).enumerate() {
        //     col = col.push(
        //         row![
        //             text(format!("{:08X}", chunk_index * 16)).font(Font {
        //                 family: Family::Monospace,
        //                 weight: Weight::Bold,
        //                 ..Default::default()
        //             }),
        //             text(format!("{:08X}", chunk[0])).font(Font {
        //                 family: Family::Monospace,
        //                 ..Default::default()
        //             }),
        //             text(format!("{:08X}", chunk[1])).font(Font {
        //                 family: Family::Monospace,
        //                 ..Default::default()
        //             }),
        //             text(format!("{:08X}", chunk[2])).font(Font {
        //                 family: Family::Monospace,
        //                 ..Default::default()
        //             }),
        //             text(format!("{:08X}", chunk[3])).font(Font {
        //                 family: Family::Monospace,
        //                 ..Default::default()
        //             }),
        //         ]
        //         .spacing(4),
        //     );
        // }

        col.into()
    }
}
