use crate::emu::event::Event;

#[derive(Default)]
pub struct MemoryWidget {
    pub memory: Vec<u32>,
}

impl MemoryWidget {
    pub fn update(&mut self, event: &Event) {
        if let Event::Update {
            memory: Some(memory),
            ..
        } = event
        {
            self.memory = memory.clone();
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Memory")
            .default_open(true)
            .show(ui, |ui| {
                for (chunk_index, chunk) in self.memory.chunks(4).enumerate() {
                    let addr = chunk_index * 16;

                    let w0 = chunk.get(0).copied().unwrap_or(0);
                    let w1 = chunk.get(1).copied().unwrap_or(0);
                    let w2 = chunk.get(2).copied().unwrap_or(0);
                    let w3 = chunk.get(3).copied().unwrap_or(0);

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{:08X}", addr))
                                .monospace()
                                .strong(),
                        );
                        ui.monospace(format!("{:08X} {:08X} {:08X} {:08X}", w0, w1, w2, w3));
                    });
                }
            });
    }
}
