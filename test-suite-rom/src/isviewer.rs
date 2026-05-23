use n64_specs::isviewer as specs;

use crate::io;

pub fn write(text: &str) {
    if text.len() == 0 {
        return;
    }

    io::wait_for_pi();

    for chunk in text.as_bytes().chunks(specs::BUFFER_SIZE) {
        for (i, bytes) in chunk.chunks(4).enumerate() {
            let word = u32::from_be_bytes([
                *bytes.get(0).unwrap_or(&0),
                *bytes.get(1).unwrap_or(&0),
                *bytes.get(2).unwrap_or(&0),
                *bytes.get(3).unwrap_or(&0),
            ]);

            io::write_uncached(specs::BUFFER_START_ADDRESS + (i * 4) as u32, word);
            io::wait_for_pi();
        }

        io::write_uncached(specs::LENGTH_ADDRESS, chunk.len() as u32);
        io::wait_for_pi();
    }
}
